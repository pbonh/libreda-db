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

//! Traits for netlist data types.

use std::hash::Hash;


/// Most basic trait of a netlist.
pub trait NetlistTrait {
    /// Type for names of circuits, instances, pins, etc.
    type NameType: Eq + Hash;
    /// Type for pin definitions.
    type PinType;
    /// Pin identifier type.
    type PinId: Eq + Hash + Clone;
    /// Pin instance identifier type.
    /// A pin instance is a pin of a circuit instance.
    type PinInstId: Eq + Hash + Clone;
    /// Either a pin or a pin instance ID.
    type TerminalId: Eq + Hash;
    // + From<Self::PinId> + From<Self::PinInstId>;
    /// Circuit identifier type.
    type CircuitId: Eq + Hash;
    /// Circuit instance identifier type.
    type CircuitInstId: Eq + Hash;
    /// Net identifier type.
    type NetId: Eq + Hash + Clone;


    /// Create a new empty netlist.
    fn new() -> Self;

    /// Create a new and empty circuit.
    fn create_circuit(&mut self, name: Self::NameType, pins: Vec<Self::NameType>) -> Self::CircuitId;

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
    fn create_net(&mut self, parent: Self::CircuitId,
                  name: Option<Self::NameType>) -> Self::NetId;

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

    /// Get the internal net attached to this pin.
    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId>;

    /// Get the external net attached to this pin instance.
    fn net_of_pin_instance(&self, pin_instance: &Self::PinInstId) -> Option<Self::NetId>;

    // /// Call a function on each circuit of the netlist.
    // fn for_each_circuit<F>(&self, f: F) where F: FnOnce<&Self::CircuidId>;

    /// Iterate over all circuits.
    fn each_circuit<'a>(&'a self) -> Box<dyn Iterator<Item=&Self::CircuitId> + 'a>;

    /// Iterate over all pins of a circuit.
    fn each_pin<'a>(&'a self, circuit: &Self::CircuitId) -> Box<dyn Iterator<Item=&Self::PinId> + 'a>;

    /// Iterate over all pin instances of a circuit.
    fn each_pin_instance<'a>(&'a self, circuit_instance: &Self::CircuitInstId) -> Box<dyn Iterator<Item=&Self::PinInstId> + 'a>;

    /// Iterate over all external nets connected to the circuit instance.
    fn each_external_net<'a>(&'a self, circuit_instance: &Self::CircuitInstId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_pin_instance(circuit_instance)
            .flat_map(move |pin_id| self.net_of_pin_instance(pin_id)))
    }

    /// Iterate over all pins of a net.
    fn each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=&Self::PinId> + 'a>;

    /// Iterate over all pins of a net.
    fn each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=&Self::PinInstId> + 'a>;

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
        let terminals: Vec<_> = self.each_pin_of_net(old_net).cloned().collect();
        // Connect each terminal to the new net.
        for pin in terminals {
            self.connect_pin(&pin, Some(new_net.clone()));
        }
        // Get terminals connected to the old net.
        let terminals: Vec<_> = self.each_pin_instance_of_net(old_net).cloned().collect();
        // Connect each terminal to the new net.
        for pin in terminals {
            self.connect_pin_instance(&pin, Some(new_net.clone()));
        }

        // Remove the now unused old net.
        self.remove_net(old_net);
    }
}