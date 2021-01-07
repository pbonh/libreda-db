/*
 * Copyright (c) 2020-2021 Thomas Kramer.
 *
 * This file is part of LibrEDA 
 * (see https://codeberg.org/libreda).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

//! Traits for netlist data types.

use std::hash::Hash;
use crate::netlist::direction::Direction;
use std::borrow::Borrow;
use std::collections::HashMap;


/// Most basic trait of a netlist.
pub trait NetlistBase {
    /// Type for names of circuits, instances, pins, etc.
    type NameType: Eq + Hash + From<String> + Into<String> + Clone
    + Borrow<String> + Borrow<str>
    + std::fmt::Display + std::fmt::Debug;
    /// Pin identifier type.
    type PinId: Eq + Hash + Clone;
    /// Pin instance identifier type.
    /// A pin instance is a pin of a circuit instance.
    type PinInstId: Eq + Hash + Clone;
    /// Either a pin or a pin instance ID.
    type TerminalId: Eq + Hash + Clone;
    /// Circuit identifier type.
    type CircuitId: Eq + Hash + Clone;
    /// Circuit instance identifier type.
    type CircuitInstId: Eq + Hash + Clone;
    /// Net identifier type.
    type NetId: Eq + Hash + Clone;


    /// Create a new empty netlist.
    fn new() -> Self;

    /// Find a circuit by its name.
    /// Return the circuit with the given name. Returns `None` if the circuit does not exist.
    fn circuit_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CircuitId>
        where Self::NameType: Borrow<N>;

    /// Find a circuit instance by its name.
    /// Returns `None` if the name does not exist.
    fn circuit_instance_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CircuitId, name: &N) -> Option<Self::CircuitInstId>
        where Self::NameType: Borrow<N>;

    /// Get the ID of the template circuit of this instance.
    fn template_circuit(&self, circuit_instance: &Self::CircuitInstId) -> Self::CircuitId;

    /// Get the ID of the template pin of this pin instance.
    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId;

    /// Get the signal direction of the pin.
    fn pin_direction(&self, pin: &Self::PinId) -> Direction;

    /// Get the ID of the parent circuit of this instance.
    fn parent_circuit(&self, circuit_instance: &Self::CircuitInstId) -> Self::CircuitId;

    /// Get the ID of the parent circuit of this pin.
    fn parent_circuit_of_pin(&self, pin: &Self::PinId) -> Self::CircuitId;

    /// Get the ID of the circuit instance that holds this pin instance.
    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CircuitInstId;

    /// Get the internal net attached to this pin.
    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId>;

    /// Get the external net attached to this pin instance.
    fn net_of_pin_instance(&self, pin_instance: &Self::PinInstId) -> Option<Self::NetId>;

    /// Get the net of the logical constant zero.
    fn net_zero(&self, parent_circuit: &Self::CircuitId) -> Self::NetId;

    /// Get the net of the logical constant one.
    fn net_one(&self, parent_circuit: &Self::CircuitId) -> Self::NetId;

    /// Find a net by its name inside the parent circuit.
    /// Returns `None` if no such net can be found.
    fn net_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CircuitId, name: &N) -> Option<Self::NetId>
        where Self::NameType: Borrow<N>;

    /// Get the name of the net.
    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType>;

    /// Get the name of the circuit.
    fn circuit_name(&self, circuit: &Self::CircuitId) -> Self::NameType;

    /// Get the name of the circuit instance.
    fn circuit_instance_name(&self, circuit_inst: &Self::CircuitInstId) -> Option<Self::NameType>;

    /// Call a function on each circuit of the netlist.
    fn for_each_circuit<F>(&self, f: F) where F: FnMut(Self::CircuitId) -> ();

    /// Get a `Vec` of all circuit IDs in this netlist.
    fn each_circuit_vec(&self) -> Vec<Self::CircuitId> {
        let mut v = Vec::new();
        self.for_each_circuit(|c| v.push(c.clone()));
        v
    }

    /// Iterate over all circuits.
    fn each_circuit<'a>(&'a self) -> Box<dyn Iterator<Item=Self::CircuitId> + 'a> {
        Box::new(self.each_circuit_vec().into_iter())
    }

    /// Call a function on each instance in this circuit.
    fn for_each_instance<F>(&self, circuit: &Self::CircuitId, f: F) where F: FnMut(Self::CircuitInstId) -> ();

    /// Get a `Vec` of the IDs of all instances in this circuit.
    fn each_instance_vec(&self, circuit: &Self::CircuitId) -> Vec<Self::CircuitInstId> {
        let mut v = Vec::new();
        self.for_each_instance(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all instances in a circuit.
    fn each_instance<'a>(&'a self, circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::CircuitInstId> + 'a> {
        Box::new(self.each_instance_vec(circuit).into_iter())
    }

    /// Iterate over all circuits that are childs of this `circuit`.
    fn each_circuit_dependency<'a>(&'a self, circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::CircuitId> + 'a>;

    /// Iterate over all circuits that hold instances of this `circuit`.
    fn each_dependent_circuit<'a>(&'a self, circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::CircuitId> + 'a>;


    /// Call a function for each pin of the circuit.
    fn for_each_pin<F>(&self, circuit: &Self::CircuitId, f: F) where F: FnMut(Self::PinId) -> ();

    /// Get a `Vec` with the IDs of all pins of this circuit.
    fn each_pin_vec(&self, circuit: &Self::CircuitId) -> Vec<Self::PinId> {
        let mut v = Vec::new();
        self.for_each_pin(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a circuit.
    fn each_pin<'a>(&'a self, circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.each_pin_vec(circuit).into_iter())
    }


    /// Call a function for each pin instance of the circuit instance.
    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CircuitInstId, f: F) where F: FnMut(Self::PinInstId) -> ();

    /// Get a `Vec` with the IDs of all pin instance of this circuit instance.
    fn each_pin_instance_vec(&self, circuit_instance: &Self::CircuitInstId) -> Vec<Self::PinInstId> {
        let mut v = Vec::new();
        self.for_each_pin_instance(circuit_instance, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pin instances of a circuit.
    fn each_pin_instance<'a>(&'a self, circuit_instance: &Self::CircuitInstId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.each_pin_instance_vec(circuit_instance).into_iter())
    }


    /// Iterate over all external nets connected to the circuit instance.
    fn each_external_net<'a>(&'a self, circuit_instance: &Self::CircuitInstId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_pin_instance(circuit_instance)
            .flat_map(move |pin_id| self.net_of_pin_instance(&pin_id)))
    }

    /// Call a function for net of the circuit.
    fn for_each_internal_net<F>(&self, circuit: &Self::CircuitId, f: F) where F: FnMut(Self::NetId) -> ();

    /// Get a `Vec` with all nets in this circuit.
    fn each_internal_net_vec(&self, circuit: &Self::CircuitId) -> Vec<Self::NetId> {
        let mut v = Vec::new();
        self.for_each_internal_net(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all defined nets inside a circuit.
    fn each_internal_net<'a>(&'a self, circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_internal_net_vec(circuit).into_iter())
    }

    /// Get the number of pins that are connected to this net.
    fn num_net_pins(&self, net: &Self::NetId) -> usize {
        let mut n = 0;
        self.for_each_pin_of_net(net, |_| n += 1);
        n
    }

    /// Get the number of pin instances that are connected to this net.
    fn num_net_pin_instances(&self, net: &Self::NetId) -> usize {
        let mut n = 0;
        self.for_each_pin_instance_of_net(net, |_| n += 1);
        n
    }

    /// Get the number of terminals that are connected to this net.
    fn num_net_terminals(&self, net: &Self::NetId) -> usize {
        self.num_net_pins(net) + self.num_net_pin_instances(net)
    }

    /// Call a function for each pin connected to this net.
    fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> ();

    /// Get a `Vec` with all pin IDs connected to this net.
    fn each_pin_of_net_vec(&self, net: &Self::NetId) -> Vec<Self::PinId> {
        let mut v = Vec::new();
        self.for_each_pin_of_net(net, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a net.
    fn each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.each_pin_of_net_vec(net).into_iter())
    }


    /// Call a function for each pin instance connected to this net.
    fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> ();

    /// Get a `Vec` with all pin instance IDs connected to this net.
    fn each_pin_instance_of_net_vec(&self, net: &Self::NetId) -> Vec<Self::PinInstId> {
        let mut v = Vec::new();
        self.for_each_pin_instance_of_net(net, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a net.
    fn each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.each_pin_instance_of_net_vec(net).into_iter())
    }
}

/// Trait for netlists that support editing.
pub trait NetlistEdit
    where Self: NetlistBase {
    /// Create a new and empty circuit.
    fn create_circuit(&mut self, name: Self::NameType, pins: Vec<(Self::NameType, Direction)>) -> Self::CircuitId;

    /// Delete the given circuit if it exists.
    fn remove_circuit(&mut self, circuit_id: &Self::CircuitId);


    /// Create a new circuit instance.
    fn create_circuit_instance(&mut self,
                               parent_circuit: &Self::CircuitId,
                               template_circuit: &Self::CircuitId,
                               name: Option<Self::NameType>) -> Self::CircuitInstId;

    /// Remove circuit instance if it exists.
    fn remove_circuit_instance(&mut self, id: &Self::CircuitInstId);

    /// Create a net net that lives in the `parent` circuit.
    fn create_net(&mut self, parent: &Self::CircuitId,
                  name: Option<Self::NameType>) -> Self::NetId;

    /// Set a new name for the net. This might panic if the name already exists.
    fn rename_net(&mut self, parent_circuit: &Self::CircuitId,
                  net_id: &Self::NetId,
                  new_name: Option<Self::NameType>);

    /// Delete the net if it exists and disconnect all connected terminals.
    fn remove_net(&mut self, net: &Self::NetId);

    /// Connect a pin to a net.
    /// Returns the old connected net, if any.
    fn connect_pin(&mut self, pin: &Self::PinId, net: Option<Self::NetId>) -> Option<Self::NetId>;

    /// Disconnect the pin from any connected net.
    /// Returns the old connected net, if any.
    fn disconnect_pin(&mut self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.connect_pin(pin, None)
    }

    /// Connect a pin instance to a net.
    /// Returns the old connected net, if any.
    fn connect_pin_instance(&mut self, pin: &Self::PinInstId, net: Option<Self::NetId>) -> Option<Self::NetId>;

    /// Disconnect the pin instance from any connected net.
    /// Returns the old connected net, if any.
    fn disconnect_pin_instance(&mut self, pin_instance: &Self::PinInstId) -> Option<Self::NetId> {
        self.connect_pin_instance(pin_instance, None)
    }

    /// Take all terminals that are connected to `old_net` and connect them to `new_net` instead.
    /// The old net is no longer used and removed.
    ///
    /// This is a default implementation that can possibly be implemented more efficiently for a concrete
    /// netlist type.
    fn replace_net(&mut self, old_net: &Self::NetId, new_net: &Self::NetId) {
        // Check that the nets live in this circuit.
        // TODO:
        // assert!(old_net.parent_circuit().ptr_eq(&self.self_reference()));
        // assert!(new_net.parent_circuit().ptr_eq(&self.self_reference()));
        // assert!(self.nets.borrow().contains_key(&old_net.id), "Old net does not exist in this circuit.");
        // assert!(self.nets.borrow().contains_key(&new_net.id), "New net does not exist in this circuit.");

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

    /// Replace the circuit instance with its contents. Remove the circuit instance afterwards.
    /// Does not purge nets nor unconnected instances.
    /// So there could be unconnected nets or unconnected instances.
    ///
    /// Nets keep their names if possible. If the net name already exists in this circuit, the name will
    /// be set to `None`.
    ///
    /// The content of the circuit instance will be renamed by appending the names like a path.
    fn flatten_circuit_instance(&mut self, circuit_instance: &Self::CircuitInstId) {
        // assert!(self.contains_instance(circuit_instance),
        //         "Instance does not live in this circuit.");

        // Get the template circuit.
        let template = self.template_circuit(circuit_instance);
        let parent_circuit = self.parent_circuit(circuit_instance);

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
                    if let Some(_) = netlist.net_by_name(&parent_circuit, &net_name) {
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

        // Copy all sub instances into this circuit.
        // And connect their pins to copies of the original nets.
        let all_instances = self.each_instance_vec(&template);
        for sub in all_instances {
            let sub_template = self.template_circuit(&sub);

            let new_name = if let (Some(sub_instance_name), Some(inst_name)) =
            (self.circuit_instance_name(&sub), self.circuit_instance_name(circuit_instance)) {
                // Construct name for the new sub instance.
                // Something like: INSTANCE_TO_BE_FLATTENED:SUB_INSTANCE{_COUNTER}
                {
                    let mut new_name = format!("{}:{}", inst_name, sub_instance_name);
                    let mut i = 0;
                    // It is possible that the instance name already exists in this circuit.
                    // If this name too already exists, append a counter.
                    while self.circuit_instance_by_name(&parent_circuit, &new_name).is_some() {
                        new_name = format!("{}:{}_{}", inst_name, sub_instance_name, i);
                        i += 1;
                    }
                    Some(new_name)
                }
            } else {
                None
            };
            let new_inst = self.create_circuit_instance(&sub_template, &sub_template,
            new_name.map(|n| n.into()));

            // Re-connect pins to copies of the original nets.
            // Loop over old/new pin instances.
            let pin_mapping: Vec<_> = self.each_pin_instance(&sub)
                .zip(self.each_pin_instance(&new_inst))
                .collect();
            for (old_pin, new_pin) in pin_mapping {
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
            // First create a the mapping from inner nets to outer nets.
            // This is necessary for the case when multiple internal pins are connected to the same
            // internal net.
            let net_replacement_mapping: HashMap<_, _> = self.each_pin_instance_vec(circuit_instance)
                .into_iter()
                .filter_map(|old_pin| {
                    let outer_old_net = self.net_of_pin_instance(&old_pin);
                    let inner_old_net = self.net_of_pin(&self.template_pin(&old_pin));
                    // If the pin was either not connected on the outside or not connected inside, nothing
                    // needs to be done.
                    if let (Some(outer_net), Some(inner_old_net)) = (outer_old_net, inner_old_net) {
                        // Get the new copy of the inner net.
                        let inner_new_net = get_new_net(self, &inner_old_net);
                        // Attach the new inner net to the outer net (by replacement).
                        Some((inner_new_net, outer_net))
                    } else {
                        // Inner and outer pin were not connected, nothing needs to be done.
                        None
                    }
                })
                .collect();
            // Make the net replacement.
            net_replacement_mapping.iter()
                .for_each(|(inner_new_net, outer_net)|
                    self.replace_net(inner_new_net, outer_net)
                );
        }


        // Remove old instance.
        self.remove_circuit_instance(circuit_instance);

        // TODO: Clean-up.
        // self.purge_nets();
        // Remove unconnected instances.
    }

    /// Delete all unconnected nets in this circuit.
    /// Return number of purged nets.
    fn purge_nets_in_circuit(&mut self, circuit_id: &Self::CircuitId) -> usize {
        let mut unused = Vec::new();
        self.for_each_internal_net(circuit_id, |n| {
            if self.num_net_terminals(&n) == 0 {
                unused.push(n)
            }
        });

        unused.iter()
            .for_each(|n| self.remove_net(n));

        return unused.len()
    }

    /// Delete all unconnected nets in all circuits.
    /// Return number of purged nets.
    fn purge_nets(&mut self) -> usize {
        let all_circuits = self.each_circuit_vec();
        all_circuits.iter()
            .map(|c| self.purge_nets_in_circuit(c))
            .sum()
    }
}