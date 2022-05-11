// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Utility functions for dealing with netlists.

use crate::traits::{NetlistBase, NetlistEdit};

use std::collections::{HashMap, HashSet};
use std::borrow::Borrow;
use std::fmt;

use itertools::Itertools;

/// Non-modifying utility functions for netlists.
/// Import the this trait to use the utility functions all types that implement the `NetlistBase` trait.
pub trait NetlistUtil: NetlistBase {
    /// Check if the net is either the constant LOW or HIGH.
    fn is_constant_net(&self, net: &Self::NetId) -> bool {
        let parent = self.parent_cell_of_net(net);
        net == &self.net_zero(&parent) || net == &self.net_one(&parent)
    }

    /// Get all nets that are connected to the circuit instance.
    fn nets_of_cell_instance(&self, inst: &Self::CellInstId) -> Box<dyn Iterator<Item=Self::NetId> + '_> {
        Box::new(self.each_pin_instance(inst)
            .flat_map(move |p| self.net_of_pin_instance(&p)))
    }

    /// Visit all circuit instances connected to this net.
    /// An instance is touched not more than once.
    fn for_each_circuit_instance_of_net<F>(&self, net: &Self::NetId, mut f: F) where F: FnMut(Self::CellInstId) -> () {
        let mut visited = HashSet::new();
        self.for_each_pin_instance_of_net(net, |pin_inst| {
            let inst = self.parent_of_pin_instance(&pin_inst);
            if !visited.contains(&inst) {
                f(inst);
            } else {
                visited.insert(inst);
            }
        })
    }

    /// Iterate over all circuit instances connected to this net.
    /// An instance is touched not more than once.
    fn each_circuit_instance_of_net_vec(&self, net: &Self::NetId) -> Vec<Self::CellInstId> {
        let mut v = Vec::new();
        self.for_each_circuit_instance_of_net(net, |c| v.push(c.clone()));
        v
    }

    /// Write the netlist in a human readable form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let circuits = self.each_cell_vec();
        // circuits.sort_by_key(|c| c.id());
        for c in &circuits {
            let circuit_name = self.cell_name(c);
            let circuit_instances = self.each_cell_instance_vec(c);
            // circuit_instances.sort_by_key(|c| c.id());

            // Get pin names together with the net they are connected to.
            let pin_names = self.each_pin(c)
                .map(|pin| {
                    let pin_name = self.pin_name(&pin);
                    let net = self.net_of_pin(&pin);
                    let net_name: Option<String> = net.
                        map(|n| self.net_name(&n)
                            .map(|n| n.into())
                            .unwrap_or("<unnamed>".into())); // TODO: Create a name.
                    format!("{}={:?}", pin_name, net_name)
                }).join(" ");

            writeln!(f, ".subckt {} {}", circuit_name, pin_names)?;
            for inst in &circuit_instances {
                // fmt::Debug::fmt(Rc::deref(&c), f)?;
                let sub_name: String = self.cell_instance_name(inst)
                    .map(|n| n.into())
                    .unwrap_or("<unnamed>".into()); // TODO: Create a name.
                let sub_template = self.template_cell(inst);
                let template_name = self.cell_name(&sub_template);
                let nets = self.each_pin_instance(inst)
                    .map(|p| {
                        let pin = self.template_pin(&p);
                        let pin_name = self.pin_name(&pin);
                        let net = self.net_of_pin_instance(&p);
                        let net_name: Option<String> = net.
                            map(|n| self.net_name(&n)
                                .map(|n| n.into())
                                .unwrap_or("<unnamed>".into()));
                        format!("{}={:?}", pin_name, net_name)
                    }).join(" ");
                writeln!(f, "    X{} {} {}", sub_name, template_name, nets)?;
            }
            writeln!(f, ".ends {}\n", circuit_name)?;
        }
        fmt::Result::Ok(())
    }

}

impl<N: NetlistBase> NetlistUtil for N {}

/// Modifying utility functions for netlists.
/// Import the this trait to use the utility functions all types that implement the `NetlistBase` trait.
pub trait NetlistEditUtil: NetlistEdit {
    /// Take all terminals that are connected to `old_net` and connect them to `new_net` instead.
    /// The old net is no longer used and removed.
    ///
    /// This is a default implementation that can possibly be implemented more efficiently for a concrete
    /// netlist type.
    fn replace_net(&mut self, old_net: &Self::NetId, new_net: &Self::NetId) {
        if old_net == new_net {
            // Nothing needs to be done.
        } else {
            // Check that the nets live in this circuit.
            assert_eq!(self.parent_cell_of_net(old_net), self.parent_cell_of_net(new_net),
                       "Old net and new net must live in the same cell.");

            // Get terminals connected to the old net.
            let terminals: Vec<_> = self.each_pin_of_net(&old_net).collect();
            // Connect each terminal to the new net.
            for pin in terminals {
                self.connect_pin(&pin, Some(new_net.clone()));
            }
            // Get terminals connected to the old net.
            let terminals: Vec<_> = self.each_pin_instance_of_net(&old_net).collect();
            // Connect each terminal to the new net.
            for pin in terminals {
                self.connect_pin_instance(&pin, Some(new_net.clone()));
            }

            // Remove the now unused old net.
            self.remove_net(&old_net);
        }
    }

    /// Replace the circuit instance with its contents. Remove the circuit instance afterwards.
    /// Does not purge nets nor unconnected instances.
    /// So there could be unconnected nets or unconnected instances.
    ///
    /// Nets keep their names if possible. If the net name already exists in this circuit, the name will
    /// be set to `None`.
    ///
    /// The content of the circuit instance will be renamed by appending the names like a path.
    fn flatten_circuit_instance(&mut self, circuit_instance: &Self::CellInstId) {

        // Get the template circuit.
        let template = self.template_cell(circuit_instance);
        let parent_circuit = self.parent_cell(circuit_instance);

        assert_ne!(template, parent_circuit, "Recursive instances are not allowed."); // Should not happen.

        // Mapping from old to new nets.
        let mut net_mapping: HashMap<Self::NetId, Self::NetId> = HashMap::new();

        // Get or create a new net as an equivalent of the old.
        let mut get_new_net = |netlist: &mut Self, old_net: &Self::NetId| -> Self::NetId {
            if let Some(net_net) = net_mapping.get(old_net) {
                net_net.clone()
            } else {
                // Get the name of the net.
                let net_name = netlist.net_name(old_net);
                // Resolve net name collisions.
                // It is possible that the net name already exists in this circuit.
                let net_name = if let Some(net_name) = net_name {
                    // Check if net name already exists.
                    if let Some(_) = netlist.net_by_name(&parent_circuit, net_name.borrow()) {
                        // Net name already exists in this circuit.
                        // Don't use it.
                        // TODO: Create a qualified name?
                        None
                    } else {
                        // Net name does not yet exist.
                        // We can use the original one.
                        Some(net_name)
                    }
                } else {
                    // No net name was given.
                    None
                };

                // Create the new net.
                let new_net = netlist.create_net(&parent_circuit, net_name);
                // Remember the mapping old_net -> net_net.
                net_mapping.insert(old_net.clone(), new_net.clone());
                new_net
            }
        };


        // Constant HIGH and LOW nets are special cases.
        // Connect previous constant nets to the equivalent constant nets of the parent cell.
        let new_net_zero = get_new_net(self, &self.net_zero(&template));
        let new_net_one = get_new_net(self, &self.net_one(&template));

        // Copy all sub instances into this circuit.
        // And connect their pins to copies of the original nets.
        let all_instances = self.each_cell_instance_vec(&template);
        for sub in all_instances {
            let sub_template = self.template_cell(&sub);

            let new_name = if let (Some(sub_instance_name), Some(inst_name)) =
            (self.cell_instance_name(&sub), self.cell_instance_name(circuit_instance)) {
                // Construct name for the new sub instance.
                // Something like: INSTANCE_TO_BE_FLATTENED:SUB_INSTANCE{_COUNTER}
                {
                    let mut new_name = format!("{}:{}", inst_name, sub_instance_name);
                    let mut i = 0;
                    // It is possible that the instance name already exists in this circuit.
                    // If this name too already exists, append a counter.
                    while self.cell_instance_by_name(&parent_circuit, &new_name).is_some() {
                        new_name = format!("{}:{}_{}", inst_name, sub_instance_name, i);
                        i += 1;
                    }
                    Some(new_name)
                }
            } else {
                None
            };
            let new_inst = self.create_cell_instance(&parent_circuit, &sub_template,
                                                     new_name.map(|n| n.into()));

            // Re-connect pins to copies of the original nets.
            // Loop over old/new pin instances.
            let pin_mapping: Vec<_> = self.each_pin_instance(&sub)
                .zip(self.each_pin_instance(&new_inst))
                .collect();
            for (old_pin, new_pin) in pin_mapping {

                // Sanity check: Ordering of the pin instances in both cell instances must be consistent.
                debug_assert_eq!(self.template_pin(&old_pin), self.template_pin(&new_pin),
                                 "Unexpected pin ordering.");

                // Get net on old pin.
                if let Some(old_net) = self.net_of_pin_instance(&old_pin) {
                    let new_net = get_new_net(self, &old_net);
                    self.connect_pin_instance(&new_pin, Some(new_net));
                }
            }
        }

        // Connect the newly created sub-instances and nets to this circuit
        // according to the old connections to the instance which is about to be flattened.
        {
            // First create the mapping from inner nets to outer nets.
            // This is necessary for the case when multiple internal pins are connected to the same
            // internal net.
            let mut net_replacement_mapping: HashMap<_, _> =
                self.each_pin_instance_vec(circuit_instance)
                    .into_iter()
                    .filter_map(|old_pin| {
                        let outer_net = self.net_of_pin_instance(&old_pin);
                        let inner_old_net = self.net_of_pin(&self.template_pin(&old_pin));
                        // If the pin was either not connected on the outside or not connected inside, nothing
                        // needs to be done.
                        if let (Some(outer_net), Some(inner_old_net)) = (outer_net, inner_old_net) {
                            // Get the new copy of the inner net.
                            let inner_new_net = get_new_net(self, &inner_old_net);
                            // Attach the new inner net to the outer net (by replacement).
                            if inner_new_net != outer_net {
                                Some((inner_new_net, outer_net))
                            } else {
                                // If the nets are the same, no replacement is necessary.
                                // This can happen for LOW and HIGH nets, since the mapping
                                // is already correct.
                                None
                            }
                        } else {
                            // Inner and outer pin were not connected, nothing needs to be done.
                            None
                        }
                    })
                    .collect();

            // Handle special cases: LOW and HIGH nets.
            net_replacement_mapping.insert(new_net_zero, self.net_zero(&parent_circuit));
            net_replacement_mapping.insert(new_net_one, self.net_one(&parent_circuit));

            // Make the net replacement.
            net_replacement_mapping.iter()
                .for_each(|(inner_new_net, outer_net)|
                    self.replace_net(inner_new_net, outer_net)
                );
            //
            // // Handle special cases: LOW and HIGH nets.
            // self.replace_net(&new_net_zero, &self.net_zero(&parent_circuit));
            // self.replace_net(&new_net_one, &self.net_one(&parent_circuit));
        }


        // Remove old instance.
        self.remove_cell_instance(circuit_instance);

        // TODO: Clean-up.
        // self.purge_nets();
        // Remove unconnected instances.
    }

    /// Flatten all instances of this circuit by replacing them with their content.
    /// Remove the circuit from the netlist afterwards.
    /// For top level circuits this is equivalent to removing them.
    fn flatten_circuit(&mut self, circuit: &Self::CellId) {
        // TODO: Assert that the circuit lives in this netlist.
        // Get all instances of the circuit.

        // Flatten all instances of the circuit.
        for r in self.each_cell_reference_vec(circuit) {
            self.flatten_circuit_instance(&r);
        }

        debug_assert_eq!(self.each_cell_reference(circuit).count(), 0,
                         "Circuit should not have any references anymore.");

        // Remove the cell.
        self.remove_cell(circuit);
    }

    /// Delete all unconnected nets in this circuit.
    /// Return number of purged nets.
    fn purge_nets_in_circuit(&mut self, circuit_id: &Self::CellId) -> usize {
        let high = self.net_one(circuit_id);
        let low = self.net_zero(circuit_id);
        let mut unused = Vec::new();
        self.for_each_internal_net(circuit_id, |n| {
            // Purge floating nets but keep the constant-value nets.
            if self.num_net_terminals(&n) == 0 && n != high && n != low {
                unused.push(n)
            }
        });

        unused.iter()
            .for_each(|n| self.remove_net(n));

        return unused.len();
    }

    /// Delete all unconnected nets in all circuits.
    /// Return number of purged nets.
    fn purge_nets(&mut self) -> usize {
        let all_circuits = self.each_cell_vec();
        all_circuits.iter()
            .map(|c| self.purge_nets_in_circuit(c))
            .sum()
    }

    /// Create names for all unnamed nets in the specified circuit.
    /// The names will consist of the `prefix` and an appended number.
    /// After calling this method, no net inside this circuit will be unnamed.
    fn create_net_names_in_circuit(&mut self, circuit_id: &Self::CellId, prefix: &str) {
        let all_nets = self.each_internal_net_vec(circuit_id);
        let unnamed_nets: Vec<_> = all_nets.iter()
            .filter(|net| self.net_name(net).is_none())
            .collect();

        if !unnamed_nets.is_empty() {

            let mut id_counter = 0;
            for unnamed_net in unnamed_nets {
                // Generate a new name.
                let net_name = loop {
                    let net_name = format!("{}{}", prefix, id_counter);
                    id_counter += 1;
                    if self.net_by_name(circuit_id, &net_name).is_none() {
                        break net_name;
                    }
                };

                let old_name = self.rename_net(unnamed_net, Some(net_name.into()));
                debug_assert!(old_name.is_none());
            }
        }
    }
}

impl<N: NetlistEdit + ?Sized> NetlistEditUtil for N {}