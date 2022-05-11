// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::traits::{HierarchyBase, NetlistBase, NetlistEdit};
use crate::prelude::{Direction, TerminalId};
use crate::decorator::{Decorator, MutDecorator};

/// Define the same functions as [`NetlistBase`] but just prepend a `d_` to
/// avoid naming conflicts.
/// The default implementation just forwards the call to the `base()`.
/// This allows to selectively re-implement some functions or fully delegate
/// the trait to an attribute of a struct.
pub trait NetlistBaseDecorator: Decorator
    where Self::D: NetlistBase
{
    // Neater solution, but only for unstable rust yet:
    // type PinId = <<Self as Decorator>::D as NetlistBase>::PinId;
    // type PinInstId = <<Self as Decorator>::D as NetlistBase>::PinInstId;

    // type NetId: Eq + Hash + Clone + std::fmt::Debug;

    fn d_template_pin(&self, pin_instance: &<Self::D as NetlistBase>::PinInstId) -> <Self::D as NetlistBase>::PinId {
        self.base().template_pin(pin_instance)
    }

    fn d_pin_direction(&self, pin: &<Self::D as NetlistBase>::PinId) -> Direction {
        self.base().pin_direction(pin)
    }

    fn d_pin_name(&self, pin: &<Self::D as NetlistBase>::PinId) -> <Self::D as HierarchyBase>::NameType {
        self.base().pin_name(pin)
    }

    fn d_pin_by_name(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId, name: &str) -> Option<<Self::D as NetlistBase>::PinId> {
        self.base().pin_by_name(parent_circuit, name)
    }

    fn d_parent_cell_of_pin(&self, pin: &<Self::D as NetlistBase>::PinId) -> <Self::D as HierarchyBase>::CellId {
        self.base().parent_cell_of_pin(pin)
    }

    fn d_parent_of_pin_instance(&self, pin_inst: &<Self::D as NetlistBase>::PinInstId) -> <Self::D as HierarchyBase>::CellInstId {
        self.base().parent_of_pin_instance(pin_inst)
    }

    fn d_pin_instance(&self, cell_inst: &<Self::D as HierarchyBase>::CellInstId, pin: &<Self::D as NetlistBase>::PinId) -> <Self::D as NetlistBase>::PinInstId {
        self.base().pin_instance(cell_inst, pin)
    }

    fn d_parent_cell_of_net(&self, net: &<Self::D as NetlistBase>::NetId) -> <Self::D as HierarchyBase>::CellId {
        self.base().parent_cell_of_net(net)
    }

    fn d_net_of_pin(&self, pin: &<Self::D as NetlistBase>::PinId) -> Option<<Self::D as NetlistBase>::NetId> {
        self.base().net_of_pin(pin)
    }

    fn d_net_of_pin_instance(&self, pin_instance: &<Self::D as NetlistBase>::PinInstId) -> Option<<Self::D as NetlistBase>::NetId> {
        self.base().net_of_pin_instance(pin_instance)
    }

    fn d_net_of_terminal(&self, terminal: &TerminalId<Self::D>) -> Option<<Self::D as NetlistBase>::NetId> {
        self.base().net_of_terminal(terminal)
    }

    fn d_net_zero(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId) -> <Self::D as NetlistBase>::NetId {
        self.base().net_zero(parent_circuit)
    }

    fn d_net_one(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId) -> <Self::D as NetlistBase>::NetId {
        self.base().net_one(parent_circuit)
    }

    fn d_net_by_name(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId, name: &str) -> Option<<Self::D as NetlistBase>::NetId> {
        self.base().net_by_name(parent_circuit, name)
    }

    fn d_net_name(&self, net: &<Self::D as NetlistBase>::NetId) -> Option<<Self::D as HierarchyBase>::NameType> {
        self.base().net_name(net)
    }


    fn d_for_each_pin<F>(&self, circuit: &<Self::D as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinId) -> () {
        self.base().for_each_pin(circuit, f)
    }

    fn d_each_pin_vec(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> Vec<<Self::D as NetlistBase>::PinId> {
        self.base().each_pin_vec(circuit)
    }

    fn d_each_pin<'a>(&'a self, circuit: &<Self::D as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinId> + 'a> {
        self.base().each_pin(circuit)
    }

    fn d_for_each_pin_instance<F>(&self, circuit_inst: &<Self::D as HierarchyBase>::CellInstId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinInstId) -> () {
        self.base().for_each_pin_instance(circuit_inst, f)
    }

    fn d_each_pin_instance_vec(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Vec<<Self::D as NetlistBase>::PinInstId> {
        self.base().each_pin_instance_vec(circuit_instance)
    }

    fn d_each_pin_instance<'a>(&'a self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinInstId> + 'a> {
        self.base().each_pin_instance(circuit_instance)
    }

    fn d_each_external_net<'a>(&'a self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::NetId> + 'a> {
        self.base().each_external_net(circuit_instance)
    }

    fn d_for_each_external_net<F>(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId, f: F)
        where F: FnMut(<Self::D as NetlistBase>::NetId) {
        self.base().for_each_external_net(circuit_instance, f)
    }

    fn d_each_external_net_vec(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Vec<<Self::D as NetlistBase>::NetId> {
        self.base().each_external_net_vec(circuit_instance)
    }

    fn d_for_each_internal_net<F>(&self, circuit: &<Self::D as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::D as NetlistBase>::NetId) -> () {
        self.base().for_each_internal_net(circuit, f)
    }

    fn d_each_internal_net_vec(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> Vec<<Self::D as NetlistBase>::NetId> {
        self.base().each_internal_net_vec(circuit)
    }

    fn d_each_internal_net<'a>(&'a self, circuit: &<Self::D as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::NetId> + 'a> {
        self.base().each_internal_net(circuit)
    }

    fn d_num_internal_nets(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> usize {
        self.base().num_internal_nets(circuit)
    }

    fn d_num_net_pins(&self, net: &<Self::D as NetlistBase>::NetId) -> usize {
        self.base().num_net_pins(net)
    }

    fn d_num_net_pin_instances(&self, net: &<Self::D as NetlistBase>::NetId) -> usize {
        self.base().num_net_pin_instances(net)
    }

    fn d_num_net_terminals(&self, net: &<Self::D as NetlistBase>::NetId) -> usize {
        self.base().num_net_terminals(net)
    }

    fn d_num_pins(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> usize {
        self.base().num_pins(circuit)
    }

    fn d_for_each_pin_of_net<F>(&self, net: &<Self::D as NetlistBase>::NetId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinId) -> () {
        self.base().for_each_pin_of_net(net, f)
    }

    fn d_each_pin_of_net_vec(&self, net: &<Self::D as NetlistBase>::NetId) -> Vec<<Self::D as NetlistBase>::PinId> {
        self.base().each_pin_of_net_vec(net)
    }

    fn d_each_pin_of_net<'a>(&'a self, net: &<Self::D as NetlistBase>::NetId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinId> + 'a> {
        self.base().each_pin_of_net(net)
    }


    fn d_for_each_pin_instance_of_net<F>(&self, net: &<Self::D as NetlistBase>::NetId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinInstId) -> () {
        self.base().for_each_pin_instance_of_net(net, f)
    }

    fn d_each_pin_instance_of_net_vec(&self, net: &<Self::D as NetlistBase>::NetId) -> Vec<<Self::D as NetlistBase>::PinInstId> {
        self.base().each_pin_instance_of_net_vec(net)
    }

    fn d_each_pin_instance_of_net<'a>(&'a self, net: &<Self::D as NetlistBase>::NetId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinInstId> + 'a> {
        self.base().each_pin_instance_of_net(net)
    }

    fn d_for_each_terminal_of_net<F>(&self, net: &<Self::D as NetlistBase>::NetId, f: F)
        where F: FnMut(TerminalId<Self::D>) -> () {
        self.base().for_each_terminal_of_net(net, f)
    }

    fn d_each_terminal_of_net_vec(&self, net: &<Self::D as NetlistBase>::NetId) -> Vec<TerminalId<Self::D>> {
        self.base().each_terminal_of_net_vec(net)
    }

    fn d_each_terminal_of_net<'a>(&'a self, net: &<Self::D as NetlistBase>::NetId)
                                  -> Box<dyn Iterator<Item=TerminalId<Self::D>> + 'a> {
        self.base().each_terminal_of_net(net)
    }
}

impl<T, N> NetlistBase for T
    where
        T: HierarchyBase<NameType=N::NameType, CellId=N::CellId, CellInstId=N::CellInstId>
        + NetlistBaseDecorator<D=N>,
        N: NetlistBase + 'static
{
    type PinId = N::PinId;
    type PinInstId = N::PinInstId;
    type NetId = N::NetId;

    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId {
        self.d_template_pin(pin_instance)
    }

    fn pin_direction(&self, pin: &Self::PinId) -> Direction {
        self.d_pin_direction(pin)
    }

    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType {
        self.d_pin_name(pin)
    }

    fn pin_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::PinId> {
        self.d_pin_by_name(parent_circuit, name)
    }

    fn parent_cell_of_pin(&self, pin: &Self::PinId) -> Self::CellId {
        self.d_parent_cell_of_pin(pin)
    }

    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CellInstId {
        self.d_parent_of_pin_instance(pin_inst)
    }

    fn parent_cell_of_net(&self, net: &Self::NetId) -> Self::CellId {
        self.d_parent_cell_of_net(net)
    }

    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.d_net_of_pin(pin)
    }

    fn net_of_pin_instance(&self, pin_instance: &Self::PinInstId) -> Option<Self::NetId> {
        self.d_net_of_pin_instance(pin_instance)
    }

    fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.d_net_zero(parent_circuit)
    }

    fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.d_net_one(parent_circuit)
    }

    fn net_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::NetId> {
        self.d_net_by_name(parent_circuit, name)
    }

    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType> {
        self.d_net_name(net)
    }

    fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> () {
        self.d_for_each_pin(circuit, f)
    }

    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CellInstId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.d_for_each_pin_instance(circuit_inst, f)
    }

    fn for_each_internal_net<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::NetId) -> () {
        self.d_for_each_internal_net(circuit, f)
    }

    fn num_pins(&self, circuit: &Self::CellId) -> usize {
        self.d_num_pins(circuit)
    }

    fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> () {
        self.d_for_each_pin_of_net(net, f)
    }

    fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.d_for_each_pin_instance_of_net(net, f)
    }
}

pub trait NetlistEditDecorator: MutDecorator
    where Self::D: NetlistBase + NetlistEdit {
    /// Create a new pin in this cell.
    /// Also adds the pin to all instances of the cell.
    fn d_create_pin(&mut self, cell: &<Self::D as HierarchyBase>::CellId, name: <Self::D as HierarchyBase>::NameType, direction: Direction) -> <Self::D as NetlistBase>::PinId {
        self.mut_base().create_pin(cell, name, direction)
    }

    /// Remove the pin from this circuit and from all instances of this circuit.
    fn d_remove_pin(&mut self, id: &<Self::D as NetlistBase>::PinId) {
        self.mut_base().remove_pin(id)
    }

    /// Change the name of the pin, returns the old name.
    /// # Panics
    /// Panics when the name is already occupied.
    fn d_rename_pin(&mut self, pin: &<Self::D as NetlistBase>::PinId, new_name: <Self::D as HierarchyBase>::NameType) -> <Self::D as HierarchyBase>::NameType {
        self.mut_base().rename_pin(pin, new_name)
    }

    /// Create a net net that lives in the `parent` circuit.
    fn d_create_net(&mut self, parent: &<Self::D as HierarchyBase>::CellId,
                    name: Option<<Self::D as HierarchyBase>::NameType>) -> <Self::D as NetlistBase>::NetId {
        self.mut_base().create_net(parent, name)
    }

    /// Set a new name for the net. This might panic if the name already exists.
    /// Returns the old name.
    fn d_rename_net(&mut self, net_id: &<Self::D as NetlistBase>::NetId,
                    new_name: Option<<Self::D as HierarchyBase>::NameType>) -> Option<<Self::D as HierarchyBase>::NameType> {
        self.mut_base().rename_net(net_id, new_name)
    }

    /// Delete the net if it exists and disconnect all connected terminals.
    fn d_remove_net(&mut self, net: &<Self::D as NetlistBase>::NetId) {
        self.mut_base().remove_net(net)
    }

    /// Connect a pin to a net.
    /// Returns the old connected net, if any.
    fn d_connect_pin(&mut self, pin: &<Self::D as NetlistBase>::PinId, net: Option<<Self::D as NetlistBase>::NetId>) -> Option<<Self::D as NetlistBase>::NetId> {
        self.mut_base().connect_pin(pin, net)
    }

    /// Connect a pin instance to a net.
    /// Returns the old connected net, if any.
    fn d_connect_pin_instance(&mut self, pin: &<Self::D as NetlistBase>::PinInstId, net: Option<<Self::D as NetlistBase>::NetId>) -> Option<<Self::D as NetlistBase>::NetId> {
        self.mut_base().connect_pin_instance(pin, net)
    }
}