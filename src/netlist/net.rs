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
//! A net represents an electric potential such as the one provided by a metal wire.

use super::netlist::*;
use super::terminal_ref::*;
use super::circuit::Circuit;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashSet;
use crate::netlist::circuit_instance::CircuitInstance;
use crate::netlist::pin::Pin;
use crate::netlist::pin_instance::PinInstance;
use genawaiter::rc::Gen;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Copied from KLayout: A single net.
/// A net connects multiple pins or terminals together. Pins are either pin or subcircuits of outgoing pins of the circuit the net lives in. Terminals are connections made to specific terminals of devices.
/// Net objects are created inside a circuit with Circuit#create_net.
/// To connect a net to an outgoing pin of a circuit, use Circuit#connect_pin, to disconnect a net from an outgoing pin use Circuit#disconnect_pin. To connect a net to a pin of a subcircuit, use SubCircuit#connect_pin, to disconnect a net from a pin of a subcircuit, use SubCircuit#disconnect_pin. To connect a net to a terminal of a device, use Device#connect_terminal, to disconnect a net from a terminal of a device, use Device#disconnect_terminal.
#[derive(Debug)]
pub struct Net {
    /// ID of the net. This uniquely identifies the net within the parent circuit.
    pub(super) id: NetIndex,
    /// Name of the net.
    pub(super) name: RefCell<Option<String>>,
    /// The circuit where the net lives in.
    /// A weak reference is needed here to avoid reference cycles.
    pub(super) parent_circuit: Weak<Circuit>,
    /// All pins that are connected to this net.
    pub(super) pins: RefCell<HashSet<Rc<Pin>>>,
    /// All pin instances that are connected to this net.
    pub(super) pin_instances: RefCell<HashSet<Rc<PinInstance>>>,
}

impl Net {
    /// Return the name of this net.
    pub fn name(&self) -> Option<String> {
        self.name.borrow().as_ref().cloned()
    }

    /// Return the name of this net or if it is `None` create a name
    /// based on the net ID.
    /// TODO: There is no guarantee yet that this name is collision free.
    pub fn create_name(&self) -> String {
        self.name()
            .unwrap_or_else(|| {
                format!("__{}", self.id.index)
            })
    }

    /// Return a qualified name of this net of the form `CIRCUIT_NAME:NET_NAME`.
    /// TODO: There is no guarantee yet that this name is collision free.
    pub fn create_qname(&self) -> String {
        format!("{}:{}", self.parent_circuit().upgrade().unwrap().name(), self.create_name())
    }

    /// Rename the net
    /// # Panics
    /// * Panics if there is already a net with this name.
    pub fn rename<S: Into<String>>(&self, name: Option<S>) {
        self.parent_circuit().upgrade()
            .unwrap()
            .rename_net(self.id(), name)
    }

    /// Get the index of this net.
    /// The index uniquely identifies this net within the circuit.
    pub fn id(&self) -> NetIndex {
        self.id
    }

    /// Get borrowed reference to the set of pins.
    pub fn pins(&self) -> impl Deref<Target=HashSet<Rc<Pin>>> + '_ {
        self.pins.borrow()
    }

    /// Get borrowed reference to the set of pin instances.
    pub fn pins_instances(&self) -> impl Deref<Target=HashSet<Rc<PinInstance>>> + '_ {
        self.pin_instances.borrow()
    }

    /// Iterate over all pins and pin instances connected to this net.
    pub fn each_terminal(&self) -> impl Iterator<Item=TerminalRef> + '_ {
        let generator = Gen::new(|co| async move {
            for t in self.pins.borrow().iter().cloned() {
                co.yield_(TerminalRef::Pin(t)).await;
            }
            for t in self.pin_instances.borrow().iter().cloned() {
                co.yield_(TerminalRef::PinInstance(t)).await;
            }
        });
        generator.into_iter()
    }

    /// Iterate over all pin instances connected to this net.
    pub fn each_pin_instance(&self) -> impl Iterator<Item=Rc<PinInstance>> + '_ {
        let generator = Gen::new(|co| async move {
            for t in self.pin_instances.borrow().iter().cloned()
            {
                co.yield_(t).await;
            }
        });
        generator.into_iter()
    }

    /// Iterate over all pins connected to this net.
    pub fn each_pin(&self) -> impl Iterator<Item=Rc<Pin>> + '_ {
        let generator = Gen::new(|co| async move {
            for p in self.pins.borrow().iter().cloned()
            {
                co.yield_(p).await;
            }
        });
        generator.into_iter()
    }

    /// Find each circuit instance that is connected to this net.
    pub fn each_instance(&self) -> impl Iterator<Item=Rc<CircuitInstance>> + '_ {
        let generator = Gen::new(|co| async move {
            // For all terminals take the ones that are connected to another sub-circuit instance.
            for p in self.pin_instances.borrow().iter() {
                if let Some(inst) = p.circuit_instance().upgrade() {
                    // Yield the sub-circuit instance that is connected to this net.
                    co.yield_(inst).await;
                } else {
                    debug_assert!(false, "Weak reference to circuit instance should never by `None`.");
                }
            }
        });
        generator.into_iter()
    }

    /// Get the circuit where this net lives in.
    pub fn parent_circuit(&self) -> Weak<Circuit> {
        self.parent_circuit.clone()
    }

    /// Return the number of pins connected to this net.
    pub fn num_pins(&self) -> usize { self.pins.borrow().len() }

    /// Return the number of pin instances connected to this net.
    pub fn num_pin_instances(&self) -> usize { self.pin_instances.borrow().len() }

    /// Return the number of terminals (pins and pin instances) connected to this net.
    pub fn num_terminals(&self) -> usize {
        self.num_pins() + self.num_pin_instances()
    }
}

impl Eq for Net {}

impl PartialEq for Net {
    /// A net is considered equal if the ID and parent circuit are identical.
    fn eq(&self, other: &Self) -> bool {
        debug_assert_ne!(self.parent_circuit.upgrade(), None,
                         "Cannot compare nets that don't live in a circuit.");
        self.id == other.id
            && self.parent_circuit.ptr_eq(&other.parent_circuit)
    }
}

impl Hash for Net {
    fn hash<H: Hasher>(&self, state: &mut H) {
        debug_assert_ne!(self.parent_circuit.upgrade(), None,
                         "Cannot hash nets that don't live in a circuit.");
        self.id.hash(state);
        self.parent_circuit.upgrade().hash(state);
    }
}