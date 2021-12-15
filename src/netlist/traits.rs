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
//!
//! Instead of putting a netlist data structure into the center of the world,
//! this data base concentrates on the way *how* a netlist can be accessed and modified.
//! The basic necessary operations are defined in the [`NetlistBase'] trait and in the
//! [`NetlistEdit`] trait.
//!
//! More complex operations on netlist are provided by the [`NetlistUtil`] and [`NetlistEditUtil`] traits.
//!
//! [`NetlistBase`]: NetlistBase
//! [`NetlistEdit`]: NetlistEdit
//! [`NetlistUtil`]: crate::netlist::util::NetlistUtil
//! [`NetlistEditUtil`]: crate::netlist::util::NetlistEditUtil

use std::hash::{Hash, Hasher};
use crate::netlist::direction::Direction;
pub use crate::traits::{HierarchyBase, HierarchyEdit};


/// A terminal is a generalization of pins and pin instances.
#[derive(Debug)]
pub enum TerminalId<N: NetlistBase + ?Sized> {
    /// Terminal is a pin.
    PinId(N::PinId),
    /// Terminal is a pin instance.
    PinInstId(N::PinInstId),
}

impl<N: NetlistBase> Hash for TerminalId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TerminalId::PinId(p) => p.hash(state),
            TerminalId::PinInstId(p) => p.hash(state)
        }
    }
}

impl<N: NetlistBase + ?Sized> Eq for TerminalId<N> {}

impl<N: NetlistBase + ?Sized> PartialEq for TerminalId<N> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PinId(p1), Self::PinId(p2)) => p1 == p2,
            (Self::PinInstId(p1), Self::PinInstId(p2)) => p1 == p2,
            (_, _) => false
        }
    }
}

impl<N: NetlistBase + ?Sized> Clone for TerminalId<N>
    where N::PinId: Clone, N::PinInstId: Clone {
    fn clone(&self) -> Self {
        match self {
            TerminalId::PinId(p) => Self::PinId(p.clone()),
            TerminalId::PinInstId(p) => Self::PinInstId(p.clone()),
        }
    }
}

/// Most basic trait for traversing a netlist.
/// A netlist extends the `HierarchyBase` and hence is hierarchical.
/// `NetlistBase` extends the components of the hierarchy with pins and nets.
/// Each cell can have pins. Each cell instance has pin instances that correspond one-to-one
/// to the pins of the template cell. Cells can contain nets. Each pin and each pin instance can be
/// connected to one or zero nets. A net can be connected to an arbitrary number of pins and pin instances.
///
/// Pins must have a name and also a signal direction.
///
/// Nets *can* have a name.
///
pub trait NetlistBase: HierarchyBase {
    /// Pin identifier type. Uniquely identifies a pin in the whole netlist.
    type PinId: Eq + Hash + Clone + std::fmt::Debug;
    /// Pin instance identifier type. Uniquely identifies a pin instance in the whole netlist.
    /// A pin instance is a pin of a circuit instance.
    type PinInstId: Eq + Hash + Clone + std::fmt::Debug;
    /// Net identifier type. Uniquely identifies a net in the whole netlist.
    type NetId: Eq + Hash + Clone + std::fmt::Debug;

    /// Get the ID of the template pin of this pin instance.
    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId;

    /// Get the signal direction of the pin.
    fn pin_direction(&self, pin: &Self::PinId) -> Direction;

    /// Get the name of the pin.
    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType;

    /// Find a pin by its name.
    /// Returns `None` if no such pin can be found.
    fn pin_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::PinId>;

    /// Get the ID of the parent circuit of this pin.
    fn parent_cell_of_pin(&self, pin: &Self::PinId) -> Self::CellId;

    /// Get the ID of the circuit instance that holds this pin instance.
    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CellInstId;

    /// Get the ID of a pin instance given the cell instance and the pin ID.
    fn pin_instance(&self, cell_inst: &Self::CellInstId, pin: &Self::PinId) -> Self::PinInstId {
        // Inefficient default implementation.
        self.each_pin_instance(cell_inst)
            .find(|inst| &self.template_pin(inst) == pin)
            .expect("No such pin found in this cell.")
    }


    /// Get the ID of the parent circuit of this net.
    fn parent_cell_of_net(&self, net: &Self::NetId) -> Self::CellId;

    /// Get the internal net attached to this pin.
    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId>;

    /// Get the external net attached to this pin instance.
    fn net_of_pin_instance(&self, pin_instance: &Self::PinInstId) -> Option<Self::NetId>;

    /// Get the net that is attached to this terminal.
    fn net_of_terminal(&self, terminal: &TerminalId<Self>) -> Option<Self::NetId> {
        match terminal {
            TerminalId::PinId(p) => self.net_of_pin(p),
            TerminalId::PinInstId(p) => self.net_of_pin_instance(p),
        }
    }

    /// Get the net of the logical constant zero.
    fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId;

    /// Get the net of the logical constant one.
    fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId;

    /// Find a net by its name inside the parent circuit.
    /// Returns `None` if no such net can be found.
    fn net_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::NetId>;

    /// Get the name of the net.
    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType>;


    /// Call a function for each pin of the circuit.
    fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> ();

    /// Get a `Vec` with the IDs of all pins of this circuit.
    fn each_pin_vec(&self, circuit: &Self::CellId) -> Vec<Self::PinId> {
        let mut v = Vec::new();
        self.for_each_pin(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a circuit.
    fn each_pin<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.each_pin_vec(circuit).into_iter())
    }

    /// Call a function for each pin instance of the circuit instance.
    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CellInstId, f: F) where F: FnMut(Self::PinInstId) -> ();

    /// Get a `Vec` with the IDs of all pin instance of this circuit instance.
    fn each_pin_instance_vec(&self, circuit_instance: &Self::CellInstId) -> Vec<Self::PinInstId> {
        let mut v = Vec::new();
        self.for_each_pin_instance(circuit_instance, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pin instances of a circuit.
    fn each_pin_instance<'a>(&'a self, circuit_instance: &Self::CellInstId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.each_pin_instance_vec(circuit_instance).into_iter())
    }

    /// Iterate over all external nets connected to the circuit instance.
    /// A net might appear more than once.
    fn each_external_net<'a>(&'a self, circuit_instance: &Self::CellInstId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_pin_instance(circuit_instance)
            .flat_map(move |pin_id| self.net_of_pin_instance(&pin_id)))
    }

    /// Iterate over all external nets connected to the circuit instance.
    /// A net might appear more than once.
    fn for_each_external_net<F>(&self, circuit_instance: &Self::CellInstId, mut f: F)
        where F: FnMut(Self::NetId) {
        self.for_each_pin_instance(circuit_instance, |i| {
            self.net_of_pin_instance(&i).iter()
                .cloned()
                .for_each(|n| f(n))
        });
    }

    /// Get a vector of all external nets connected to the circuit instance.
    /// A net might appear more than once.
    fn each_external_net_vec(&self, circuit_instance: &Self::CellInstId) -> Vec<Self::NetId> {
        let mut v = Vec::new();
        self.for_each_external_net(circuit_instance, |n| v.push(n.clone()));
        v
    }

    /// Call a function for net of the circuit.
    fn for_each_internal_net<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::NetId) -> ();

    /// Get a `Vec` with all nets in this circuit.
    fn each_internal_net_vec(&self, circuit: &Self::CellId) -> Vec<Self::NetId> {
        let mut v = Vec::new();
        self.for_each_internal_net(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all defined nets inside a circuit.
    fn each_internal_net<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_internal_net_vec(circuit).into_iter())
    }

    /// Return the number of nets defined inside a cell.
    fn num_internal_nets(&self, circuit: &Self::CellId) -> usize {
        // Inefficient default implementation.
        let mut counter = 0;
        self.for_each_internal_net(circuit, |_| counter += 1);
        counter
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

    /// Get the number of pins of a circuit.
    fn num_pins(&self, circuit: &Self::CellId) -> usize;

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

    /// Call a function for each terminal connected to this net.
    fn for_each_terminal_of_net<F>(&self, net: &Self::NetId, mut f: F)
        where F: FnMut(TerminalId<Self>) -> () {
        self.for_each_pin_of_net(net, |p| f(TerminalId::PinId(p)));
        self.for_each_pin_instance_of_net(net, |p| f(TerminalId::PinInstId(p)));
    }

    /// Get a `Vec` with all terminal IDs connected to this net.
    fn each_terminal_of_net_vec(&self, net: &Self::NetId) -> Vec<TerminalId<Self>> {
        let mut v = Vec::new();
        self.for_each_terminal_of_net(net, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all terminals of a net.
    fn each_terminal_of_net<'a>(&'a self, net: &Self::NetId)
                                -> Box<dyn Iterator<Item=TerminalId<Self>> + 'a> {
        Box::new(self.each_terminal_of_net_vec(net).into_iter())
    }
}


/// Trait for netlists that support editing.
///
/// This includes:
///
/// * creation and removal of pins and nets
/// * connecting pins and pin instances to nets
/// * renaming nets
/// * renaming pins
///
/// More complex operations which can be build on top of the basic operations
/// are provided by the [`NetlistEditUtil`] trait.
///
/// [`NetlistEditUtil`]: crate::netlist::util::NetlistEditUtil
pub trait NetlistEdit: NetlistBase + HierarchyEdit {

    // /// Create a multi-bit port.
    // /// Internally creates a pin for every bit of the port.
    // fn create_bus(&mut self, circuit: &Self::CellId, name: Self::NameType, direction: Direction, width: usize) -> Vec<Self::PinId>;

    /// Create a new pin in this cell.
    /// Also adds the pin to all instances of the cell.
    fn create_pin(&mut self, cell: &Self::CellId, name: Self::NameType, direction: Direction) -> Self::PinId;

    /// Remove the pin from this circuit and from all instances of this circuit.
    fn remove_pin(&mut self, id: &Self::PinId);

    /// Change the name of the pin, returns the old name.
    /// # Panics
    /// Panics when the name is already occupied.
    fn rename_pin(&mut self, pin: &Self::PinId, new_name: Self::NameType) -> Self::NameType;

    /// Create a net net that lives in the `parent` circuit.
    fn create_net(&mut self, parent: &Self::CellId,
                  name: Option<Self::NameType>) -> Self::NetId;

    /// Set a new name for the net. This might panic if the name already exists.
    /// Returns the old name.
    fn rename_net(&mut self, net_id: &Self::NetId,
                  new_name: Option<Self::NameType>) -> Option<Self::NameType>;

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


    /// Connect a terminal to a net.
    /// Returns the old connected net, if any.
    fn connect_terminal(&mut self, terminal: &TerminalId<Self>, net: Option<Self::NetId>) -> Option<Self::NetId> {
        match terminal {
            TerminalId::PinId(p) => self.connect_pin(p, net),
            TerminalId::PinInstId(p) => self.connect_pin_instance(p, net),
        }
    }

    /// Disconnect the terminal from any connected net.
    /// Returns the old connected net, if any.
    fn disconnect_terminal(&mut self, terminal: &TerminalId<Self>) -> Option<Self::NetId> {
        self.connect_terminal(terminal, None)
    }

}