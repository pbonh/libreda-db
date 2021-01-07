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
//! Alternative netlist implementation. Not currently used, nor complete.

use std::collections::{HashSet, HashMap};
use itertools::Itertools;
use std::borrow::Borrow;
use std::hash::Hash;
use super::traits::NetlistBase;
use crate::netlist::direction::Direction;
use crate::netlist::traits::NetlistEdit;
use crate::rc_string::RcString;
use std::fmt::Debug;

/// Circuit identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CircuitId(usize);

/// Circuit instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CircuitInstId(usize);

/// Pin identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PinId(usize);

/// Pin instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PinInstId(usize);

/// Either a pin or pin instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TerminalId {
    /// Terminal is a pin.
    Pin(PinId),
    /// Terminal is a pin instance.
    PinInst(PinInstId),
}

impl From<PinId> for TerminalId {
    fn from(id: PinId) -> Self {
        TerminalId::Pin(id)
    }
}

impl From<PinInstId> for TerminalId {
    fn from(id: PinInstId) -> Self {
        TerminalId::PinInst(id)
    }
}

/// Net identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NetId(usize);

/// A circuit is defined by an interface (pins) and
/// a content which consists of interconnected circuit instances.
#[derive(Debug, Clone)]
pub struct Circuit {
    /// Name of the circuit.
    pub name: RcString,
    /// Pin definitions.
    pub pins: Vec<PinId>,
    /// Instances inside this circuit.
    pub instances: HashSet<CircuitInstId>,
    /// Circuit instances that reference to this circuit.
    pub references: HashSet<CircuitInstId>,
    /// All circuits that have instances of this circuit.
    pub parents: HashSet<CircuitId>,
    /// Nets IDs stored by name.
    nets_by_name: HashMap<RcString, NetId>,
    /// Logic constant LOW net.
    net_low: NetId,
    /// Logic constant HIGH net.
    net_high: NetId,
}

/// Instance of a circuit.
#[derive(Debug, Clone)]
pub struct CircuitInst {
    /// The ID of the template circuit.
    pub circuit: CircuitId,
    /// The ID of the parent circuit where this instance lives in.
    pub parent: CircuitId,
    /// List of pins of this instance.
    pub pins: Vec<PinInstId>,
}

/// Single bit wire pin.
#[derive(Debug, Clone)]
pub struct Pin {
    /// Name of the pin.
    pub name: RcString,
    /// Signal type/direction of the pin.
    pub direction: Direction,
    /// Parent circuit of this pin.
    pub circuit: CircuitId,
    /// Net that is connected to this pin.
    pub net: Option<NetId>,
}

/// Instance of a pin.
#[derive(Debug, Clone)]
pub struct PinInst {
    /// ID of the template pin.
    pub pin: PinId,
    /// Circuit instance where this pin instance lives in.
    pub circuit_inst: CircuitInstId,
    /// Net connected to this pin instance.
    pub net: Option<NetId>,
}

/// A net represents an electric potential or a wire.
#[derive(Debug, Clone)]
pub struct Net {
    /// Name of the net.
    pub name: Option<RcString>,
    /// Parent circuit of the net.
    pub parent: CircuitId,
    /// Pins connected to this net.
    pub pins: HashSet<PinId>,
    /// Pin instances connected to this net.
    pub pin_instances: HashSet<PinInstId>,
}

/// A netlist is the container of circuits.
#[derive(Debug, Default)]
pub struct HashMapNetlist {
    circuits: HashMap<CircuitId, Circuit>,
    circuits_by_name: HashMap<RcString, CircuitId>,
    circuit_instances: HashMap<CircuitInstId, CircuitInst>,
    nets: HashMap<NetId, Net>,
    pins: HashMap<PinId, Pin>,
    pin_instances: HashMap<PinInstId, PinInst>,

    id_counter_circuit: usize,
    id_counter_circuit_inst: usize,
    id_counter_pin: usize,
    id_counter_pin_inst: usize,
    id_counter_net: usize,
}

impl HashMapNetlist {
    /// Get a circuit reference by its ID.
    pub fn circuit(&self, id: &CircuitId) -> &Circuit {
        &self.circuits[id]
    }

    /// Get a mutable reference to the circuit by its ID.
    fn circuit_mut(&mut self, id: &CircuitId) -> &mut Circuit {
        self.circuits.get_mut(id).unwrap()
    }

    /// Get a reference to a circuit instance.
    pub fn circuit_inst(&self, id: &CircuitInstId) -> &CircuitInst {
        &self.circuit_instances[id]
    }

    /// Get a reference to a net by its ID.
    pub fn net(&self, id: &NetId) -> &Net {
        &self.nets[id]
    }

    /// Get a mutable reference to a net by its ID.
    fn net_mut(&mut self, id: &NetId) -> &mut Net {
        self.nets.get_mut(id).unwrap()
    }

    /// Get a reference to a pin by its ID.
    pub fn pin(&self, id: &PinId) -> &Pin {
        &self.pins[id]
    }

    /// Get a mutable reference to a pin by its ID.
    fn pin_mut(&mut self, id: &PinId) -> &mut Pin {
        self.pins.get_mut(id).unwrap()
    }

    /// Get a reference to a pin instance by its ID.
    pub fn pin_inst(&self, id: &PinInstId) -> &PinInst {
        &self.pin_instances[id]
    }

    /// Get a mutable reference to a pin instance by its ID.
    fn pin_inst_mut(&mut self, id: &PinInstId) -> &mut PinInst {
        self.pin_instances.get_mut(id).unwrap()
    }

    /// Get the value of a counter and increment the counter afterwards.
    fn next_id_counter(ctr: &mut usize) -> usize {
        let c = *ctr;
        *ctr += 1;
        c
    }

    /// Append a new pin to the `parent` circuit.
    fn create_pin(&mut self, parent: CircuitId, name: RcString, direction: Direction) -> PinId {
        let id = PinId(HashMapNetlist::next_id_counter(&mut self.id_counter_pin));
        let pin = Pin {
            name,
            direction,
            circuit: parent,
            net: Default::default(),
        };
        self.pins.insert(id, pin);
        id
    }

    /// Insert a new pin instance to a circuit instance.
    fn create_pin_inst(&mut self, circuit: CircuitInstId, pin: PinId) -> PinInstId {
        let id = PinInstId(HashMapNetlist::next_id_counter(&mut self.id_counter_pin_inst));
        let pin = PinInst {
            pin: pin,
            circuit_inst: circuit,
            net: None,
        };
        self.pin_instances.insert(id, pin);
        id
    }

    /// Get all nets that are connected to the circuit instance.
    pub fn circuit_inst_nets(&self, circuit_inst_id: &CircuitInstId) -> impl Iterator<Item=NetId> + '_ {
        self.circuit_inst(circuit_inst_id).pins.iter()
            .flat_map(move |p| self.pin_inst(p).net)
    }

    /// Iterate over all pins connected to a net.
    pub fn pins_for_net(&self, net: &NetId) -> impl Iterator<Item=PinId> + '_ {
        self.net(net).pins.iter().copied()
    }

    /// Iterate over all pin instances connected to a net.
    pub fn pins_instances_for_net(&self, net: &NetId) -> impl Iterator<Item=PinInstId> + '_ {
        self.net(net).pin_instances.iter().copied()
    }

    /// Iterate over all pins and pin instances connected to a net.
    pub fn terminals_for_net(&self, net: &NetId) -> impl Iterator<Item=TerminalId> + '_ {
        self.pins_for_net(net).map(|p| TerminalId::Pin(p))
            .chain(self.pins_instances_for_net(net).map(|p| TerminalId::PinInst(p)))
    }

    /// Remove all unconnected nets.
    /// Return number of purged nets.
    pub fn purge_nets(&mut self) -> usize {
        let unconnected: Vec<_> = self.nets.iter()
            .filter(|(_, n)| n.pin_instances.len() + n.pins.len() == 0)
            .map(|(&id, _)| id)
            .collect();
        let num = unconnected.len();
        for net in unconnected {
            self.remove_net(&net);
        }
        num
    }


    /// Return number of top level circuits (roots of the circuit tree).
    pub fn top_circuit_count(&self) -> usize {
        self.circuits.values()
            .filter(|c| c.parents.len() == 0)
            .count()
    }

    /// Iterate through all nets that are defined in the netlist.
    pub fn each_net(&self) -> impl Iterator<Item=NetId> + '_ {
        self.nets.keys().copied()
    }
}

impl NetlistBase for HashMapNetlist {
    type NameType = RcString;
    type PinId = PinId;
    type PinInstId = PinInstId;
    type TerminalId = TerminalId;
    type CircuitId = CircuitId;
    type CircuitInstId = CircuitInstId;
    type NetId = NetId;

    /// Create an empty netlist.
    fn new() -> Self {
        HashMapNetlist::default()
    }

    /// Find a circuit by its name.
    fn circuit_by_name<S: ?Sized + Eq + Hash>(&self, name: &S) -> Option<CircuitId>
        where Self::NameType: Borrow<S> {
        self.circuits_by_name.get(name).copied()
    }

    fn circuit_instance_by_name<N: ?Sized + Eq + Hash>(&self, _parent_circuit: &Self::CircuitId, _name: &N)
                                                       -> Option<Self::CircuitInstId>
        where Self::NameType: Borrow<N> {
        unimplemented!()
    }

    fn template_circuit(&self, circuit_instance: &Self::CircuitInstId) -> Self::CircuitId {
        self.circuit_inst(circuit_instance).parent
    }

    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId {
        self.pin_inst(pin_instance).pin
    }

    fn pin_direction(&self, pin: &Self::PinId) -> Direction {
        self.pin(pin).direction
    }

    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType {
        self.pin(pin).name.clone()
    }

    fn parent_circuit(&self, circuit_instance: &Self::CircuitInstId) -> Self::CircuitId {
        self.circuit_inst(circuit_instance).parent
    }

    fn parent_circuit_of_pin(&self, pin: &Self::PinId) -> Self::CircuitId {
        self.pin(pin).circuit
    }

    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CircuitInstId {
        self.pin_inst(pin_inst).circuit_inst
    }


    /// Get the net connected to this pin.
    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.pin(pin).net
    }

    /// Get the net connected to this pin instance.
    fn net_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Option<Self::NetId> {
        self.pin_inst(pin_inst).net
    }

    fn net_zero(&self, parent_circuit: &Self::CircuitId) -> Self::NetId {
        self.circuit(parent_circuit).net_low
    }

    fn net_one(&self, parent_circuit: &Self::CircuitId) -> Self::NetId {
        self.circuit(parent_circuit).net_high
    }

    fn net_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CircuitId, name: &N) -> Option<Self::NetId>
        where Self::NameType: Borrow<N> {
        self.circuit(parent_circuit).nets_by_name.get(name).copied()
    }

    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType> {
        self.net(net).name.clone()
    }

    fn circuit_name(&self, circuit: &Self::CircuitId) -> Self::NameType {
        self.circuit(circuit).name.clone()
    }

    fn circuit_instance_name(&self, _circuit_inst: &Self::CircuitInstId) -> Option<Self::NameType> {
        None // Circuit instance don't have a name.
    }

    fn for_each_circuit<F>(&self, f: F) where F: FnMut(Self::CircuitId) -> () {
        self.circuits.keys().copied().for_each(f)
    }


    /// Iterate over all circuits.
    fn each_circuit(&self) -> Box<dyn Iterator<Item=CircuitId> + '_> {
        Box::new(self.circuits.keys().cloned())
    }

    fn for_each_instance<F>(&self, circuit: &Self::CircuitId, f: F) where F: FnMut(Self::CircuitInstId) -> () {
        self.circuit(circuit).instances.iter()
            .copied().for_each(f)
    }

    fn each_circuit_dependency<'a>(&'a self, _circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::CircuitId>> {
        unimplemented!()
    }

    fn each_dependent_circuit<'a>(&'a self, _circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=Self::CircuitId>> {
        unimplemented!()
    }

    fn for_each_pin<F>(&self, circuit: &Self::CircuitId, f: F) where F: FnMut(Self::PinId) -> () {
        self.circuit(circuit).pins.iter().copied().for_each(f)
    }


    /// Iterate over all pins of a circuit.
    fn each_pin(&self, circuit_id: &CircuitId) -> Box<dyn Iterator<Item=PinId> + '_> {
        Box::new(self.circuit(circuit_id).pins.iter().copied())
    }

    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CircuitInstId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.circuit_inst(circuit_inst).pins.iter().copied().for_each(f)
    }

    fn each_pin_instance<'a>(&'a self, circuit_inst: &Self::CircuitInstId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.circuit_inst(circuit_inst).pins.iter().copied())
    }

    fn for_each_internal_net<F>(&self, circuit: &Self::CircuitId, f: F) where F: FnMut(Self::NetId) -> () {
        unimplemented!()
    }

    fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> () {
        self.net(net).pins.iter().copied().for_each(f)
    }


    fn each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.net(net).pins.iter().copied())
    }

    fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.net(net).pin_instances.iter().copied().for_each(f)
    }

    fn each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.net(net).pin_instances.iter().copied())
    }
}

impl NetlistEdit for HashMapNetlist {
    /// Create a new circuit with a given list of pins.
    fn create_circuit(&mut self, name: Self::NameType, pins: Vec<(Self::NameType, Direction)>) -> CircuitId {
        assert!(!self.circuits_by_name.contains_key(&name), "Circuit with this name already exists.");
        let id = CircuitId(HashMapNetlist::next_id_counter(&mut self.id_counter_circuit));

        // Create pins.
        let pins = pins.into_iter()
            .map(|(name, direction)| self.create_pin(id, name, direction))
            .collect();

        let circuit = Circuit {
            name: name.clone(),
            pins,
            instances: Default::default(),
            references: Default::default(),
            parents: Default::default(),
            nets_by_name: Default::default(),
            // Create LOW and HIGH nets.
            net_low: NetId(0),
            net_high: NetId(0),
        };

        self.circuits.insert(id, circuit);
        self.circuits_by_name.insert(name, id);

        // Create LOW and HIGH nets.
        let net_low = self.create_net(&id, None);
        let net_high = self.create_net(&id, None);

        let c = self.circuit_mut(&id);
        c.net_low = net_low;
        c.net_high = net_high;

        id
    }

    /// Remove all instances inside the circuit,
    fn remove_circuit(&mut self, circuit_id: &CircuitId) {
        // Remove all instances inside this circuit.
        let instances = self.circuit(circuit_id).instances.iter().copied().collect_vec();
        for inst in instances {
            self.remove_circuit_instance(&inst);
        }
        // Remove all instances of this circuit.
        let references = self.circuit(circuit_id).references.iter().copied().collect_vec();
        for inst in references {
            self.remove_circuit_instance(&inst);
        }
        // Clean up pin definitions.
        let pins = self.circuit(circuit_id).pins.clone();
        for pin in pins {
            self.pins.remove(&pin).unwrap();
        }
        // Remove the circuit.
        let name = self.circuit(circuit_id).name.clone();
        self.circuits_by_name.remove(&name).unwrap();
        self.circuits.remove(&circuit_id).unwrap();
    }

    /// Create a new instance of `circuit_template` in the `parent` circuit.
    fn create_circuit_instance(&mut self, parent: &CircuitId,
                               circuit_template: &CircuitId,
                               name: Option<Self::NameType>) -> CircuitInstId {
        let id = CircuitInstId(HashMapNetlist::next_id_counter(&mut self.id_counter_circuit_inst));

        // Check that there is no cycle.
        // Find root.
        {
            let mut parents = vec![*parent];
            loop {
                if parents.is_empty() {
                    // Found a root, there is no cycle.
                    break;
                } else {
                    if parents.contains(&circuit_template) {
                        // Loop found.
                        panic!("Cannot create loops!");
                    }
                    // Find parents on next level.
                    let new_parents = parents.iter()
                        .flat_map(|p| self.circuit(p).parents.iter().copied())
                        .collect();
                    parents = new_parents;
                }
            };
        }

        // Create pin instances from template pins.
        let pins = self.circuit(&circuit_template).pins.clone()
            .iter()
            .map(|&p| self.create_pin_inst(id, p))
            .collect();

        let inst = CircuitInst {
            circuit: *circuit_template,
            parent: *parent,
            pins,
        };

        self.circuit_instances.insert(id, inst);
        self.circuit_mut(&parent).instances.insert(id);
        self.circuit_mut(&circuit_template).references.insert(id);

        id
    }


    /// Remove a circuit instance after disconnecting it from the nets.
    fn remove_circuit_instance(&mut self, circuit_inst_id: &CircuitInstId) {
        // Disconnect all pins first.
        for pin in self.circuit_inst(circuit_inst_id).pins.clone() {
            self.disconnect_pin_instance(&pin);
        }
        // Remove the instance and all references.
        let parent = self.circuit_inst(&circuit_inst_id).parent;
        let template = self.circuit_inst(&circuit_inst_id).circuit;
        self.circuit_instances.remove(&circuit_inst_id).unwrap();
        self.circuit_mut(&parent).instances.remove(circuit_inst_id);
        self.circuit_mut(&template).references.remove(circuit_inst_id);
    }

    /// Create a new net in the `parent` circuit.
    fn create_net(&mut self, parent: &CircuitId, name: Option<Self::NameType>) -> NetId {
        assert!(self.circuits.contains_key(parent));

        let id = NetId(HashMapNetlist::next_id_counter(&mut self.id_counter_net));
        let net = Net {
            name: name.clone(),
            parent: *parent,
            pins: Default::default(),
            pin_instances: Default::default(),
        };
        self.nets.insert(id, net);
        if let Some(name) = name {
            self.circuit_mut(parent).nets_by_name.insert(name, id);
        }
        id
    }

    fn rename_net(&mut self, parent_circuit: &Self::CircuitId,
                  net_id: &Self::NetId, new_name: Option<Self::NameType>) {
        assert_eq!(parent_circuit, &self.nets.get(net_id).expect("Net not found.").parent);


        // Check if a net with this name already exists.
        if let Some(name) = &new_name {
            if let Some(other) = self.circuit(parent_circuit).nets_by_name.get(name) {
                if other != net_id {
                    panic!("Net name already exists.")
                } else {
                    return;
                }
            }
        }

        let maybe_old_name = self.net_mut(net_id).name.take();

        // Remove the old name mapping.
        if let Some(old_name) = maybe_old_name {
            self.circuit_mut(parent_circuit).nets_by_name.remove(&old_name);
        }

        // Add the new name mapping.
        if let Some(new_name) = new_name {
            self.nets.get_mut(net_id)
                .expect("Net not found.")
                .name.replace(new_name.clone());
            self.circuit_mut(parent_circuit).nets_by_name.insert(new_name, *net_id);
        }
    }

    /// Disconnect all connected terminals and remove the net.
    fn remove_net(&mut self, net: &NetId) {
        let pins = self.pins_for_net(net).collect_vec();
        let pin_insts = self.pins_instances_for_net(net).collect_vec();
        let parent_circuit = self.net(net).parent;

        for p in pins {
            self.disconnect_pin(&p);
        }
        for p in pin_insts {
            self.disconnect_pin_instance(&p);
        }
        let name = self.net(&net).name.clone();
        self.nets.remove(&net).unwrap();
        if let Some(name) = &name {
            self.circuit_mut(&parent_circuit).nets_by_name.remove(name).unwrap();
        }
    }

    /// Connect the pin to a net.
    fn connect_pin(&mut self, pin: &PinId, net: Option<NetId>) -> Option<NetId> {
        if let Some(net) = net {
            assert_eq!(self.pin(&pin).circuit, self.net(&net).parent, "Pin and net do not live in the same circuit.");
            self.net_mut(&net).pins.insert(*pin);
        }
        if let Some(net) = net {
            self.pin_mut(&pin).net.replace(net)
        } else {
            self.pin_mut(&pin).net.take()
        }
    }

    /// Connect the pin to a net.
    fn connect_pin_instance(&mut self, pin: &PinInstId, net: Option<NetId>) -> Option<Self::NetId> {
        if let Some(net) = net {
            assert_eq!(self.circuit_inst(&self.pin_inst(pin).circuit_inst).parent,
                       self.net(&net).parent, "Pin and net do not live in the same circuit.");
            self.net_mut(&net).pin_instances.insert(*pin);
        }
        if let Some(net) = net {
            self.pin_inst_mut(pin).net.replace(net)
        } else {
            self.pin_inst_mut(pin).net.take()
        }
    }
}

#[test]
fn test_create_populated_netlist() {
    let mut netlist = HashMapNetlist::new();
    let top = netlist.create_circuit("TOP".into(),
                                     vec![
                                         ("A".into(), Direction::Input),
                                         ("B".into(), Direction::Output)
                                     ],
    );
    assert_eq!(Some(top), netlist.circuit_by_name("TOP"));

    let sub_a = netlist.create_circuit("SUB_A".into(),
                                       vec![
                                           ("A".into(), Direction::Input),
                                           ("B".into(), Direction::Output)
                                       ],
    );
    let sub_b = netlist.create_circuit("SUB_B".into(),
                                       vec![
                                           ("A".into(), Direction::Input),
                                           ("B".into(), Direction::Output)
                                       ],
    );

    let inst_a = netlist.create_circuit_instance(&top, &sub_a, None);
    let _inst_b = netlist.create_circuit_instance(&top, &sub_b, None);

    let net_a = netlist.create_net(&top, Some("NetA".into()));
    let net_b = netlist.create_net(&top, Some("NetB".into()));

    let pins_a = netlist.each_pin_instance(&inst_a).collect_vec();
    let pins_top = netlist.each_pin(&top).collect_vec();

    netlist.connect_pin_instance(&pins_a[0], Some(net_a));
    netlist.connect_pin_instance(&pins_a[1], Some(net_b));

    netlist.connect_pin(&pins_top[0], Some(net_a));
    netlist.connect_pin(&pins_top[1], Some(net_b));

    dbg!(&netlist);
    dbg!(netlist.terminals_for_net(&net_a).collect_vec());
}