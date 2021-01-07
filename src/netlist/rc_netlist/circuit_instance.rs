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
//! A `CircuitInstance` is an instantiation of a `Circuit` also called 'sub-circuit'.

use super::prelude::*;
use std::rc::{Weak, Rc};
use std::hash::{Hash, Hasher};

/// Represents an instantiation of a `Circuit` (a sub-circuit).
#[derive(Debug)]
pub struct CircuitInstance {
    /// Instance name
    pub(super) name: Option<String>,
    /// Circuit of which this is an instance (template).
    pub(super) circuit: Weak<Circuit>,
    /// ID of the template circuit.
    pub(super) circuit_id: CircuitIndex,
    /// Circuit where this instance lives in.
    pub(super) parent_circuit: Weak<Circuit>,
    /// ID of the parent circuit.
    pub(super) parent_circuit_id: CircuitIndex,
    /// Identifier. Uniquely identifies the instance within the parent circuit.
    pub(super) id: CircuitInstIndex,
    /// Instances of the pins defined by the circuit.
    pub(super) pin_instances: Vec<Rc<PinInstance>>,
}

impl Eq for CircuitInstance {}

impl PartialEq for CircuitInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent_circuit_id == other.parent_circuit_id
    }
}

impl Hash for CircuitInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.parent_circuit_id.hash(state);
    }
}

impl CircuitInstance {
    /// Gets the parent circuit where this instance lives in.
    pub fn parent_circuit(&self) -> Weak<Circuit> {
        self.parent_circuit.clone()
    }

    /// Gets the circuit template referenced by this instance.
    pub fn circuit_ref(&self) -> Weak<Circuit> {
        self.circuit.clone()
    }

    /// Gets the ID of the circuit template referenced by this instance.
    pub fn circuit_id(&self) -> CircuitIndex {
        self.circuit_id
    }

    /// Connect a pin on the outside of the sub circuit to a net that is defined in the parent
    /// circuit.
    ///
    /// Returns the previously connected net.
    ///
    /// # Panics
    /// Panics if the supplied net does not exist in the parent circuit.
    /// Also panics if the referenced pin does not exist.
    pub fn connect_pin_by_id(&self, pin_instance_id: usize, net: Option<Rc<Net>>) -> Option<Rc<Net>> {
        let pin = self.pin_instances.get(pin_instance_id).unwrap();
        self.connect_pin(pin, net)
    }

    /// Connect a pin on the outside of the sub circuit to a net that is defined in the parent
    /// circuit.
    ///
    /// Returns the previously connected net.
    ///
    /// # Panics
    /// * Panics if the pin instance does not live in this circuit instance.
    /// * Panics if the net does not live in the same parent circuit as this circuit instance.
    pub fn connect_pin(&self, pin: &Rc<PinInstance>, net: Option<Rc<Net>>) -> Option<Rc<Net>> {

        // Check that the pin instance lives in this circuit instance.
        assert!(self.eq(&pin.circuit_instance().upgrade().unwrap()),
                "Pin does not live in this circuit instance.");

        // Check that the net lives in this circuit.
        if let Some(net) = &net {
            assert!(net.parent_circuit.ptr_eq(&self.parent_circuit()),
                    "Net does not live in this circuit.");
        }


        if let Some(old_net) = pin.net() {
            // Remove this terminal from the old net.
            old_net.pin_instances.borrow_mut().remove(&pin.clone());
        }

        if let Some(net) = &net {
            // Add the terminal to the net.
            net.pin_instances.borrow_mut()
                .insert(pin.clone());
        }

        // Write the net into the pin instance.
        pin.net.replace(net)
    }

    /// Disconnects a pin on the outside from the attached net.
    /// Short-cut for `.connect_pin_by_id(pin_id, None)`.
    ///
    /// Returns the previously connected net.
    ///
    /// # Panics
    /// Panics if the referenced pin does not exist.
    pub fn disconnect_pin_by_id(&self, pin_instance_id: usize) -> Option<Rc<Net>> {
        self.connect_pin_by_id(pin_instance_id, None)
    }

    /// Disconnects a pin on the outside from the attached net.
    /// Short-cut for `.connect_pin(pin_instance, None)`.
    /// Returns the previously connected net.
    pub fn disconnect_pin(&self, pin_instance: &Rc<PinInstance>) -> Option<Rc<Net>> {
        self.connect_pin(pin_instance, None)
    }

    /// Iterate over all pins instances.
    pub fn each_pin_instance(&self) -> impl Iterator<Item=&Rc<PinInstance>> + ExactSizeIterator {
        self.pin_instances.iter()
    }

    /// Get a `Vec` with all pins instances.
    pub fn each_pin_instance_vec(&self) -> Vec<Rc<PinInstance>> {
        self.pin_instances.to_vec()
    }

    /// Get index of this circuit instance.
    pub fn id(&self) -> CircuitInstIndex {
        self.id
    }

    /// Get the name of this circuit instance.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Get the net connected to the referenced pin.
    ///
    /// # Panics
    /// Panics if the referenced pin does not exist.
    pub fn net_for_pin(&self, pin_index: usize) -> Option<Rc<Net>> {
        self.pin_instances[pin_index]
            .net.borrow().clone()
    }

    /// Return the number of pins.
    pub fn pin_count(&self) -> usize {
        self.pin_instances.len()
    }

    /// Get the pin instance by the pin ID.
    /// # Panics
    /// Panics if the pin with this ID does not exist.
    pub fn pin_instance_by_id(&self, pin_index: usize) -> Rc<PinInstance> {
        self.pin_instances[pin_index].clone()
    }
}