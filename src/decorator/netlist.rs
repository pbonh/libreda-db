use crate::traits::{HierarchyBase, HierarchyEdit, NetlistBase, NetlistEdit};
use crate::prelude::{Direction, TerminalId};
use crate::decorator::{Decorator, MutDecorator};
use std::hash::Hash;

/// Define the same functions as [`NetlistBase`] but just prepend a `d_` to
/// avoid naming conflicts.
/// The default implementation just forwards the call to the `base()`.
/// This allows to selectively re-implement some functions or fully delegate
/// the trait to an attribute of a struct.
pub trait NetlistBaseDecorator: Decorator
    where Self::D: NetlistBase<NetId=Self::NetId>
{
    // Neater solution, but only for unstable rust yet:
    // type PinId = <<Self as Decorator>::D as NetlistBase>::PinId;
    // type PinInstId = <<Self as Decorator>::D as NetlistBase>::PinInstId;

    type NetId: Eq + Hash + Clone + std::fmt::Debug;

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

    fn d_parent_cell_of_net(&self, net: &Self::NetId) -> <Self::D as HierarchyBase>::CellId {
        self.base().parent_cell_of_net(net)
    }

    fn d_net_of_pin(&self, pin: &<Self::D as NetlistBase>::PinId) -> Option<Self::NetId> {
        self.base().net_of_pin(pin)
    }

    fn d_net_of_pin_instance(&self, pin_instance: &<Self::D as NetlistBase>::PinInstId) -> Option<Self::NetId> {
        self.base().net_of_pin_instance(pin_instance)
    }

    fn d_net_of_terminal(&self, terminal: &TerminalId<Self::D>) -> Option<Self::NetId> {
        self.base().net_of_terminal(terminal)
    }

    fn d_net_zero(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId) -> Self::NetId {
        self.base().net_zero(parent_circuit)
    }

    fn d_net_one(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId) -> Self::NetId {
        self.base().net_one(parent_circuit)
    }

    fn d_net_by_name(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId, name: &str) -> Option<Self::NetId> {
        self.base().net_by_name(parent_circuit, name)
    }

    fn d_net_name(&self, net: &Self::NetId) -> Option<<Self::D as HierarchyBase>::NameType>{
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

    fn d_each_external_net<'a>(&'a self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        self.base().each_external_net(circuit_instance)
    }

    fn d_for_each_external_net<F>(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId, f: F)
        where F: FnMut(Self::NetId) {
        self.base().for_each_external_net(circuit_instance, f)
    }

    fn d_each_external_net_vec(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Vec<Self::NetId> {
        self.base().each_external_net_vec(circuit_instance)
    }

    fn d_for_each_internal_net<F>(&self, circuit: &<Self::D as HierarchyBase>::CellId, f: F) where F: FnMut(Self::NetId) -> () {
        self.base().for_each_internal_net(circuit, f)
    }

    fn d_each_internal_net_vec(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> Vec<Self::NetId> {
        self.base().each_internal_net_vec(circuit)
    }

    fn d_each_internal_net<'a>(&'a self, circuit: &<Self::D as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        self.base().each_internal_net(circuit)
    }

    fn d_num_internal_nets(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> usize {
        self.base().num_internal_nets(circuit)
    }

    fn d_num_net_pins(&self, net: &Self::NetId) -> usize {
        self.base().num_net_pins(net)
    }

    fn d_num_net_pin_instances(&self, net: &Self::NetId) -> usize {
        self.base().num_net_pin_instances(net)
    }

    fn d_num_net_terminals(&self, net: &Self::NetId) -> usize {
        self.base().num_net_terminals(net)
    }

    fn d_num_pins(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> usize {
        self.base().num_pins(circuit)
    }

    fn d_for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinId) -> () {
        self.base().for_each_pin_of_net(net, f)
    }

    fn d_each_pin_of_net_vec(&self, net: &Self::NetId) -> Vec<<Self::D as NetlistBase>::PinId> {
       self.base().each_pin_of_net_vec(net)
    }

    fn d_each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinId> + 'a> {
        self.base().each_pin_of_net(net)
    }


    fn d_for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinInstId) -> (){
        self.base().for_each_pin_instance_of_net(net, f)
    }

    fn d_each_pin_instance_of_net_vec(&self, net: &Self::NetId) -> Vec<<Self::D as NetlistBase>::PinInstId> {
        self.base().each_pin_instance_of_net_vec(net)
    }

    fn d_each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinInstId> + 'a> {
        self.base().each_pin_instance_of_net(net)
    }

    fn d_for_each_terminal_of_net<F>(&self, net: &Self::NetId, f: F)
        where F: FnMut(TerminalId<Self::D>) -> () {
        self.base().for_each_terminal_of_net(net, f)
    }

    fn d_each_terminal_of_net_vec(&self, net: &Self::NetId) -> Vec<TerminalId<Self::D>> {
        self.base().each_terminal_of_net_vec(net)
    }

    fn d_each_terminal_of_net<'a>(&'a self, net: &Self::NetId)
                                  -> Box<dyn Iterator<Item=TerminalId<Self::D>> + 'a> {
        self.base().each_terminal_of_net(net)
    }
}