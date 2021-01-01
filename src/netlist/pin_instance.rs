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
//!

use super::prelude::*;

use std::cell::RefCell;
use std::rc::{Weak, Rc};
use std::hash::{Hash, Hasher};

/// A `PinInstance` represents the pin of a circuit instance.
/// Each `PinInstance` corresponds to a `Pin` definition of the instantiated circuit.
pub struct PinInstance {
    /// The ID of the circuit instance where this pin instance lives in.
    pub(super) circuit_instance_id: CircuitInstIndex,
    /// A weak reference to the circuit instance where this pin instance lives in.
    pub(super) circuit_instance: RefCell<Weak<CircuitInstance>>,
    /// The pin of which this `PinInstance` is an instance.
    pub(super) pin: Rc<Pin>,
    /// The net to which this pin instance is connected to.
    pub(super) net: RefCell<Option<Rc<Net>>>,
}

impl std::fmt::Debug for PinInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let net_name = self.net.borrow().as_ref().and_then(|n| n.name());
        f.debug_struct("PinInstance")
            .field("circuit_instance_id", &self.circuit_instance_id)
            .field("pin.name()", &self.pin.name())
            .field("net.name()", &net_name)
            .finish()
    }
}

impl PinInstance {
    /// Get weak reference to the circuit instance where this pin instance lives in.
    pub fn circuit_instance(&self) -> Weak<CircuitInstance> {
        return self.circuit_instance.borrow().clone();
    }

    /// Connect the pin to the given net or disconnect it if `None` is given as a net.
    /// This is a shortcut for calling `connect_pin_by_id` on the parent circuit instance.
    ///
    /// Returns the previously connected net.
    /// TODO: Remove this and use the `connect_pin` function on the parent circuit?
    pub fn connect_net(&self, net: Option<Rc<Net>>) -> Option<Rc<Net>> {
        self.circuit_instance().upgrade()
            .expect("Cannot connect a pin instance to a net if the circuit instance does not exist anymore.")
            .connect_pin_by_id(self.id(), net)
    }

    /// Disconnect the pin from the internal net.
    /// This is a shortcut for calling `connect_pin_by_id` on the parent circuit instance.
    /// Returns the previously connected net.
    pub fn disconnect_net(&self) -> Option<Rc<Net>> {
        self.connect_net(None)
    }

    /// Get the net that is connected to this pin instance, if any.
    pub fn net(&self) -> Option<Rc<Net>> {
        self.net.borrow().clone()
    }

    /// Get the pin ID.
    pub fn id(&self) -> usize {
        self.pin.id()
    }

    /// Get the pin definition.
    pub fn pin(&self) -> &Rc<Pin> {
        &self.pin
    }
}

impl Eq for PinInstance {}

impl PartialEq for PinInstance {
    fn eq(&self, other: &Self) -> bool {
        self.circuit_instance_id == other.circuit_instance_id
            && self.pin.eq(&other.pin)
    }
}

impl Hash for PinInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.circuit_instance_id.hash(state);
        self.pin.hash(state);
    }
}