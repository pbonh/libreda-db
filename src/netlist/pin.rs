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
//! A `Pin` is a single bit connection of a circuit.

use std::cell::{RefCell, Cell};
use std::rc::{Weak, Rc};
use std::hash::{Hash, Hasher};
use std::fmt;
use crate::netlist::prelude::*;

/// Definition of a pin into a circuit.
/// TODO: Make multi-bit capable.
#[derive(Clone)]
pub struct Pin {
    /// Positional ID of the pin.
    pub(super) id: usize,
    /// ID of the circuit where this pin lives in.
    /// This will be set the moment the circuit is instantiated with the pin as argument.
    pub(super) parent_circuit_id: CircuitIndex,
    /// Reference to the circuit where this pin lives in.
    pub(super) parent_circuit: RefCell<Weak<Circuit>>,
    /// Name of the pin.
    name: String,
    /// Signal direction.rs of the pin.
    direction: Cell<Direction>,
    /// Net that is connected to this pin from within the circuit.
    pub(super) internal_net: RefCell<Option<Rc<Net>>>,
    /// Associated port and index within the port, if any.
    port: Option<(Rc<Port>, usize)>
}

impl Eq for Pin {}

impl PartialEq for Pin {
    /// Pins are considered equal if they have the same parent circuit and the same
    /// ID number.
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent_circuit_id == other.parent_circuit_id
    }
}

impl Hash for Pin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.parent_circuit_id.hash(state);
    }
}

impl fmt::Debug for Pin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pin")
            .field("name", &self.name)
            .field("direction", &self.direction)
            .field("id", &self.id)
            .field("parent_circuit_id", &self.parent_circuit_id)
            .finish()
    }
}

impl Pin {

    /// Create a new pin with a name and a direction.
    pub fn new<S: Into<String>>(name: S, direction: Direction) -> Self {
        Pin {
            id: 0,
            name: name.into(),
            direction: Cell::new(direction),
            internal_net: Default::default(),
            parent_circuit: Default::default(),
            parent_circuit_id: CircuitIndex::new(0),
            port: None
        }
    }

    /// Convenience function to create a new input pin with a name.
    pub fn new_input<S: Into<String>>(name: S) -> Self {
        Self::new(name, Direction::Input)
    }

    /// Convenience function to create a new output pin with a name.
    pub fn new_output<S: Into<String>>(name: S) -> Self {
        Self::new(name, Direction::Output)
    }

    /// Get the name of the pin.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Return the input/output direction of the pin.
    pub fn direction(&self) -> Direction {
        self.direction.get()
    }

    /// Set a new pin direction and return the old one.
    pub fn set_direction(&self, direction: Direction) -> Direction {
        self.direction.replace(direction)
    }

    /// Get the ID of the pin.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Connect the pin to the given net or disconnect it if `None` is given as a net.
    /// This is a shortcut for calling `connect_pin_by_id` on the parent circuit instance.
    /// Returns the previously connected net.
    ///
    /// # Panics
    /// Panics if the parent circuit does not exist anymore.
    /// TODO: Remove this and use the `connect_pin` function on the parent circuit?
    pub fn connect_net(&self, net: Option<Rc<Net>>) -> Option<Rc<Net>> {
        self.parent_circuit().upgrade()
            .expect("Cannot connect a pin if its parent circuit does not exist.")
            .connect_pin_by_id(self.id, net)
    }

    /// Disconnect the pin from the internal net.
    /// This is a shortcut for calling `connect_pin_by_id` on the parent circuit instance.
    /// Returns the previously connected net.
    /// TODO: Remove this and use the `connect_pin` function on the parent circuit?
    pub fn disconnect_net(&self) -> Option<Rc<Net>> {
        self.connect_net(None)
    }

    /// Get the net that is internally connected to this pin, if any.
    pub fn internal_net(&self) -> Option<Rc<Net>> {
        self.internal_net.borrow().clone()
    }

    /// Get the circuit where this net lives in.
    pub fn parent_circuit(&self) -> Weak<Circuit> {
        self.parent_circuit.borrow().clone()
    }
}