/*
 * Copyright (c) 2020-2020 Thomas Kramer.
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
//! Data structures for representation of netlists.

use super::prelude::*;
use crate::index::{Index, IndexGenerator};

use std::rc::Rc;

use std::collections::HashMap;
use std::hash::Hash;
use std::borrow::Borrow;
use std::fmt;

use itertools::Itertools;
use std::ops::Deref;

use log::debug;

/// Data type used for identifying a circuit instance (sub circuit).
pub type CircuitInstIndex = Index<CircuitInstance>;
pub(crate) type CircuitInstIndexGenerator = IndexGenerator<CircuitInstance>;

/// Data type used for identifying a net.
pub type NetIndex = Index<Net>;
pub(crate) type NetIndexGenerator = IndexGenerator<Net>;

/// Data type used for identifying a circuit.
pub type CircuitIndex = Index<Circuit>;
pub(crate) type CircuitIndexGenerator = IndexGenerator<Circuit>;

/// Collection of circuits.
pub struct Netlist {
    // /// Weak reference to the netlist itself. This must be created by the netlist wrapper.
    // self_reference: Weak<RefCell<NetlistImpl>>,
    /// Circuits defined in this circuit.
    circuits: HashMap<CircuitIndex, Rc<Circuit>>,
    /// Circuits indexed by name.
    circuits_by_name: HashMap<String, CircuitIndex>,
    /// Generator for circuit indices.
    circuit_index_generator: CircuitIndexGenerator,
}

impl fmt::Debug for Netlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut circuits = self.each_circuit().collect_vec();
        circuits.sort_by_key(|c| c.id());
        f.debug_struct("Netlist")
            .field("circuits", &circuits)
            .finish()
    }
}

impl fmt::Display for Netlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut circuits = self.each_circuit().collect_vec();
        circuits.sort_by_key(|c| c.id());
        for c in circuits {
            fmt::Display::fmt(Rc::deref(c), f)?;
        }
        fmt::Result::Ok(())
    }
}

impl Netlist {
    /// Create a new empty netlist.
    pub fn new() -> Self {
        Netlist {
            circuits: Default::default(),
            circuits_by_name: Default::default(),
            circuit_index_generator: CircuitIndexGenerator::new(1), // Start at 1 because 0 is used as 'invalid'.
        }
    }

    /// Create a new and empty circuit.
    ///
    /// # Example
    /// ```rust
    /// use libreda_db::netlist::prelude::*;
    ///
    /// let mut netlist = Netlist::new();
    /// let pins = vec![
    ///     Pin::new("Input_A", Direction::Input),
    ///     Pin::new("Output_B", Direction::Output)
    /// ];
    /// // Create a circuit with a given name and pin definition.
    /// let top = netlist.create_circuit("TOP", pins);
    /// assert_eq!(top.pin_by_id(0).unwrap().name(), "Input_A");
    /// assert_eq!(top.pin_by_id(1).unwrap().name(), "Output_B");
    /// ```
    pub fn create_circuit<S: Into<String>>(&mut self, name: S, pins: Vec<Pin>) -> Rc<Circuit> {
        let name = name.into();

        // Check that circuit with this name does not yet exist.
        if self.circuits_by_name.contains_key(&name) {
            panic!(format!("Circuit '{}' already exists!", &name));
        }

        // Create new circuit index.
        let circuit_id = self.circuit_index_generator.next();

        let circuit = Circuit::new(circuit_id, name, pins);

        // Create lookup table by name.
        self.circuits_by_name.insert(circuit.name.to_owned(), circuit_id);

        // Add circuit to the collection.
        let circuit_rc = Rc::new(circuit);
        // Store reference to the circuit itself inside the circuit.
        circuit_rc.self_reference.replace(Rc::downgrade(&circuit_rc));
        // Store reference to the circuit in its pins.
        circuit_rc.each_pin()
            .for_each(|p| { p.parent_circuit.replace(Rc::downgrade(&circuit_rc)); });

        // Store the circuit in the netlist.
        self.circuits.insert(circuit_id, circuit_rc.clone());

        // Return circuit reference.
        circuit_rc
    }


    /// Return the circuit with the given id. Returns `None` if the circuit does not exist.
    pub fn circuit_by_id(&self, id: &CircuitIndex) -> Option<Rc<Circuit>> {
        self.circuits.get(id).cloned()
    }

    /// Return the circuit with the given name. Returns `None` if the circuit does not exist.
    pub fn circuit_by_name<S: ?Sized>(&self, name: &S) -> Option<Rc<Circuit>>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.circuits_by_name.get(name)
            .and_then(|i| self.circuit_by_id(i))
    }

    /// Iterate over all circuits in this netlist.
    pub fn each_circuit(&self) -> impl Iterator<Item=&Rc<Circuit>> + ExactSizeIterator {
        self.circuits.values()
    }

    /// Iterate over all circuits in this netlist starting with leaf circuits.
    pub fn each_circuit_bottom_up(&self) -> () {
        unimplemented!()
    }

    /// Iterate over all circuits in this netlist starting with the root circuits.
    pub fn each_circuit_top_down(&self) -> () {
        unimplemented!()
    }

    /// Flatten all instances of this circuit by replacing them with their content.
    /// Remove the circuit from the netlist afterwards.
    /// For top level circuits this is equivalent to removing them.
    pub fn flatten_circuit(&mut self, circuit: &Rc<Circuit>) {
        debug!("Flatten circuit {}.", circuit.name());
        // TODO: Assert that the circuit lives in this netlist.
        // Get all instances of the circuit.
        let references: Vec<_> = circuit.references().iter().cloned().collect();
        // Flatten all instances of the circuit.
        for r in references {
            let parent = r.parent_circuit().upgrade().unwrap();
            parent.flatten_circuit_instance(&r)
        }

        debug_assert!(!circuit.has_references(), "Circuit should not have any references anymore.");

        // Remove the circuit.
        self.remove_circuit(circuit);
    }

    // /// Flatten all circuits of this netlist.
    // /// Only top level circuits will remain.
    // pub fn flatten(&mut self) {
    //     // Get all circuits.
    //     // TODO: Sort them by hierarchy for more efficient flattening.
    //     let all_circuits: Vec<_> = self.each_circuit()
    //         // Convert to weak references because some circuits might get removed
    //         // from the netlist during the flattening process.
    //         .map(Rc::downgrade)
    //         .take(1)
    //         .collect();
    //
    //     debug!("Flattening {} circuits.", all_circuits.len());
    //
    //     // Flatten all the circuits.
    //     for circuit in all_circuits {
    //         if let Some(circuit) = circuit.upgrade() {
    //             // Flatten only non-top circuits.
    //             if circuit.has_references() {
    //                 self.flatten_circuit(&circuit)
    //             }
    //         } else {
    //             debug!("Weak reference.")
    //         }
    //     }
    // }

    /// Delete all floating nets in all circuits.
    /// Return number of purged nets.
    pub fn purge_nets(&mut self) -> usize {
        self.each_circuit()
            .map(|c| c.purge_nets())
            .sum()
    }

    /// Delete the given circuit if it exists.
    pub fn remove_circuit(&mut self, circuit: &Rc<Circuit>) -> () {
        // Remove all circuit instances.
        let references = circuit.each_reference().collect_vec();
        for inst in references {
            circuit.remove_circuit_instance(&inst)
        }
        // Check that now there are no references to this circuit anymore.
        debug_assert_eq!(circuit.num_references(), 0);
        // Remove the circuit.
        self.circuits_by_name.remove(&circuit.name).unwrap();
        self.circuits.remove(&circuit.id());
    }

    /// Return number of top circuits (roots).
    pub fn top_circuit_count(&self) -> usize {
        // Count how many circuits are not referenced.
        self.each_circuit()
            .filter(|c| c.num_references() == 0)
            .count()
    }
}

impl Clone for Netlist {
    fn clone(&self) -> Self {
        // Create new netlist.
        let mut netlist = Self::new();

        // Clone all circuits.
        let circuit_map = {
            let mut circuit_map = HashMap::new();
            for old_circuit in self.each_circuit() {
                let pins = old_circuit.each_pin()
                    .map(|p| Pin::new(p.name(), p.direction()))
                    .collect();
                let new_circuit = netlist.create_circuit(old_circuit.name(), pins);
                circuit_map.insert(old_circuit.clone(), new_circuit);
            }
            circuit_map
        };

        // Clone all instances.
        for old_circuit in self.each_circuit() {
            let new_circuit = circuit_map[old_circuit].clone();

            // Clone nets.
            let net_map = {
                let mut net_map = HashMap::new();
                for old_net in old_circuit.each_net() {
                    let new_net = new_circuit.create_net(old_net.name());
                    net_map.insert(old_net, new_net);
                }
                net_map
            };

            // Connect circuit pins.
            for old_pin in old_circuit.each_pin() {
                let new_net = old_pin.internal_net()
                    .map(|n| net_map[&n].clone());
                new_circuit.connect_pin_by_id(old_pin.id, new_net);
            }

            // Clone instances and connect them to the nets.
            for old_inst in old_circuit.each_instance() {
                let new_inst = new_circuit.create_circuit_instance(
                    &circuit_map[&old_inst.circuit_ref().upgrade().unwrap()],
                    old_inst.name().unwrap(),
                );

                // Connect pins to right nets.
                for (old_pin_inst, new_pin_inst) in old_inst.each_pin_instance()
                    .zip(new_inst.each_pin_instance()) {
                    let new_net = old_pin_inst.net()
                        .map(|n| net_map[&n].clone());
                    new_pin_inst.connect_net(new_net.as_ref());
                }
            }
        }

        netlist
    }
}

#[test]
fn test_create_pin() {
    let _ = Pin::new("A", Direction::None);
    let _ = Pin::new("A".to_string(), Direction::None);
    let _ = Pin::new(&"A".to_string(), Direction::None);
}

#[test]
fn test_netlist_create_circuit() {
    let mut netlist = Netlist::new();
    let pins = vec![Pin::new("A", Direction::Input)];
    let top = netlist.create_circuit("TOP", pins);
    assert_eq!(top.each_pin().len(), 1);
    assert_eq!(netlist.top_circuit_count(), 1);
}

#[test]
fn test_netlist_remove_circuit() {
    // Remove a circuit without any instances.
    let mut netlist = Netlist::new();
    let pins = vec![Pin::new("A", Direction::Input)];
    {
        let top = netlist.create_circuit("TOP", pins);
        netlist.remove_circuit(&top);
        assert_eq!(netlist.top_circuit_count(), 0);
    }
}

#[test]
fn test_netlist_create_net() {
    let mut netlist = Netlist::new();
    let pins = vec![Pin::new("A", Direction::Input)];
    let top = netlist.create_circuit("TOP", pins);

    // Create a new net.
    let net_x = top.create_net(Some("x"));
    assert_eq!(top.net_count(), 1, "net_count() is wrong.");

    // Test if the net can be found by name.
    assert!(Rc::ptr_eq(&net_x, &top.net_by_name("x").unwrap()),
            "Failed to find net by name.");
}

#[test]
fn test_netlist_connect_pin() {
    let mut netlist = Netlist::new();
    let pins = vec![Pin::new("TOP_A", Direction::Input)];
    let top = netlist.create_circuit("TOP", pins);
    let pins = vec![Pin::new("SUB_A", Direction::Input)];
    let sub = netlist.create_circuit("SUB", pins);

    // Create a new nets.
    let net1 = top.create_net(Some("net1"));

    // Create instance of SUB.
    let inst_sub = top.create_circuit_instance(&sub, "INST_SUB1");
    // Connect pin to net1.
    inst_sub.connect_pin_by_id(0, &net1);

    assert_eq!(net1.num_terminals(), 1);
    assert_eq!(inst_sub.net_for_pin(0), Some(net1.clone()));

    // Connect net1 to the pin A of the TOP circuit.
    top.connect_pin_by_id(0, net1.clone());
    assert_eq!(net1.num_terminals(), 2);
}

#[test]
fn test_netlist_circuit_remove_net() {
    let mut netlist = Netlist::new();
    let pins = vec![Pin::new("TOP_A", Direction::Input)];
    let top = netlist.create_circuit("TOP", pins);
    let pins = vec![Pin::new("SUB_A", Direction::Input)];
    let sub = netlist.create_circuit("SUB", pins);

    // Create a new nets.
    let net1 = top.create_net(Some("net1"));

    // Create instance of SUB.
    let inst_sub = top.create_circuit_instance(&sub, "INST_SUB1");

    // Check that the circuit template is now referenced once by an instance.
    assert_eq!(sub.num_references(), 1);

    // Connect pin to net1.
    inst_sub.connect_pin_by_id(0, &net1);

    assert_eq!(net1.num_terminals(), 1);
    assert_eq!(inst_sub.net_for_pin(0), Some(net1.clone()));

    // Connect net1 to the pin A of the TOP circuit.
    top.connect_pin_by_id(0, net1.clone());
    assert_eq!(net1.num_terminals(), 2);

    top.remove_net(&net1);
    assert_eq!(net1.num_terminals(), 0);
    assert_eq!(top.net_for_pin(0), None);
    assert_eq!(inst_sub.net_for_pin(0), None);
}


#[test]
fn test_netlist_clone() {
    let netlist = {
        let mut netlist = Netlist::new();
        let pins = vec![Pin::new("TOP_A", Direction::Input)];
        let top = netlist.create_circuit("TOP", pins);
        let pins = vec![Pin::new("SUB_A", Direction::Input)];
        let sub = netlist.create_circuit("SUB", pins);

        // Create a new nets.
        let net1 = top.create_net(Some("net1"));

        // Create instance of SUB.
        let inst_sub = top.create_circuit_instance(&sub, "INST_SUB1");

        // Connect pin to net1.
        inst_sub.connect_pin_by_id(0, &net1);

        // Connect net1 to the pin A of the TOP circuit.
        top.connect_pin_by_id(0, net1.clone());

        netlist
    };

    let netlist_clone = netlist.clone();

    assert_eq!(netlist_clone.top_circuit_count(), 1);
    let top = netlist_clone.circuit_by_name("TOP").unwrap();
    let _sub = netlist_clone.circuit_by_name("SUB").unwrap();

    assert_eq!(top.net_count(), 1);
    assert_eq!(top.num_instances(), 1);

    let net1 = top.net_by_name("net1").unwrap();
    assert_eq!(net1.num_terminals(), 2);

    let inst_sub = top.circuit_instance_by_name("INST_SUB1").unwrap();

    assert_eq!(inst_sub.net_for_pin(0), Some(net1.clone()));

}