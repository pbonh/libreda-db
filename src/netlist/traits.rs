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

/// Most basic trait of a netlist.
pub trait NetlistTrait {
    /// Type for names of circuits, instances, pins, etc.
    type NameType;
    /// Type for pin definitions.
    type PinType;
    /// Pin identifier type.
    type PinId;
    /// Pin instance identifier type.
    /// A pin instance is a pin of a circuit instance.
    type PinInstId;
    /// Either a pin or a pin instance ID.
    type TerminalId: From<Self::PinId> + From<Self::PinInstId>;
    /// Circuit identifier type.
    type CircuitId;
    /// Circuit instance identifier type.
    type CircuitInstId;
    /// Net identifier type.
    type NetId;


    /// Create a new empty netlist.
    fn new() -> Self;

    /// Create a new and empty circuit.
    fn create_circuit(&mut self, name: Self::NameType, pins: Vec<Self::NameType>) -> Self::CircuitId;

    /// Delete the given circuit if it exists.
    fn remove_circuit(&mut self, circuit_id: &Self::CircuitId);

    /// Create a new circuit instance.
    fn create_circuit_instance(&mut self,
                               parent_circuit: Self::CircuitId,
                               template_circuit: Self::CircuitId) -> Self::CircuitInstId;

    /// Remove circuit instance if it exists.
    fn remove_circuit_instance(&mut self, id: &Self::CircuitInstId);

    /// Create a net net that lives in the `parent` circuit.
    fn create_net(&mut self, parent: Self::CircuitId,
                  name: Self::NameType) -> Self::NetId;

    /// Delete the net if it exists and disconnect all connected terminals.
    fn remove_net(&mut self, net: &Self::NetId);

    /// Connect a pin to a net.
    fn connect_pin(&mut self, pin: Self::PinId, net: Option<Self::NetId>);

    /// Disconnect the pin from any connected net.
    fn disconnect_pin(&mut self, pin: Self::PinId) {
        self.connect_pin(pin, None)
    }

    /// Connect a pin instance to a net.
    fn connect_pin_instance(&mut self, pin: Self::PinInstId, net: Option<Self::NetId>);

    /// Disconnect the pin instance from any connected net.
    fn disconnect_pin_instance(&mut self, pin: Self::PinInstId) {
        self.connect_pin_instance(pin, None)
    }
}