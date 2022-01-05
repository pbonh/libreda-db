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

    fn d_pin_name(&self, pin: &<Self::D as NetlistBase>::PinId) -> <Self::D as HierarchyBase>::NameType;

    fn d_pin_by_name(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId, name: &str) -> Option<<Self::D as NetlistBase>::PinId>;

    fn d_parent_cell_of_pin(&self, pin: &<Self::D as NetlistBase>::PinId) -> <Self::D as HierarchyBase>::CellId;

    fn d_parent_of_pin_instance(&self, pin_inst: &<Self::D as NetlistBase>::PinInstId) -> <Self::D as HierarchyBase>::CellInstId;

    fn d_pin_instance(&self, cell_inst: &<Self::D as HierarchyBase>::CellInstId, pin: &<Self::D as NetlistBase>::PinId) -> <Self::D as NetlistBase>::PinInstId {
        todo!()
    }


    fn d_parent_cell_of_net(&self, net: &Self::NetId) -> <Self::D as HierarchyBase>::CellId;

    fn d_net_of_pin(&self, pin: &<Self::D as NetlistBase>::PinId) -> Option<Self::NetId>;

    fn d_net_of_pin_instance(&self, pin_instance: &<Self::D as NetlistBase>::PinInstId) -> Option<Self::NetId>;

    fn d_net_of_terminal(&self, terminal: &TerminalId<Self::D>) -> Option<Self::NetId> {
        todo!()
    }

    fn d_net_zero(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId) -> Self::NetId;

    fn d_net_one(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId) -> Self::NetId;

    fn d_net_by_name(&self, parent_circuit: &<Self::D as HierarchyBase>::CellId, name: &str) -> Option<Self::NetId>;

    fn d_net_name(&self, net: &Self::NetId) -> Option<<Self::D as HierarchyBase>::NameType>;


    fn d_for_each_pin<F>(&self, circuit: &<Self::D as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinId) -> ();

    fn d_each_pin_vec(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> Vec<<Self::D as NetlistBase>::PinId> {
        todo!()
    }

    fn d_each_pin<'a>(&'a self, circuit: &<Self::D as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinId> + 'a> {
        todo!()
    }

    fn d_for_each_pin_instance<F>(&self, circuit_inst: &<Self::D as HierarchyBase>::CellInstId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinInstId) -> ();

    fn d_each_pin_instance_vec(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Vec<<Self::D as NetlistBase>::PinInstId> {
        todo!()
    }

    fn d_each_pin_instance<'a>(&'a self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinInstId> + 'a> {
        todo!()
    }

    fn d_each_external_net<'a>(&'a self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        todo!()
    }

    fn d_for_each_external_net<F>(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId, mut f: F)
        where F: FnMut(Self::NetId) {
        todo!()
    }

    fn d_each_external_net_vec(&self, circuit_instance: &<Self::D as HierarchyBase>::CellInstId) -> Vec<Self::NetId> {
        todo!()
    }

    fn d_for_each_internal_net<F>(&self, circuit: &<Self::D as HierarchyBase>::CellId, f: F) where F: FnMut(Self::NetId) -> ();

    fn d_each_internal_net_vec(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> Vec<Self::NetId> {
        todo!()
    }

    fn d_each_internal_net<'a>(&'a self, circuit: &<Self::D as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        todo!()
    }

    fn d_num_internal_nets(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> usize {
        todo!()
    }

    fn d_num_net_pins(&self, net: &Self::NetId) -> usize {
        todo!()
    }

    fn d_num_net_pin_instances(&self, net: &Self::NetId) -> usize {
        todo!()
    }

    fn d_num_net_terminals(&self, net: &Self::NetId) -> usize {
        todo!()
    }

    fn d_num_pins(&self, circuit: &<Self::D as HierarchyBase>::CellId) -> usize;

    fn d_for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinId) -> ();

    fn d_each_pin_of_net_vec(&self, net: &Self::NetId) -> Vec<<Self::D as NetlistBase>::PinId> {
        todo!()
    }

    fn d_each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinId> + 'a> {
        todo!()
    }


    fn d_for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(<Self::D as NetlistBase>::PinInstId) -> ();

    fn d_each_pin_instance_of_net_vec(&self, net: &Self::NetId) -> Vec<<Self::D as NetlistBase>::PinInstId> {
        todo!()
    }

    fn d_each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=<Self::D as NetlistBase>::PinInstId> + 'a> {
        todo!()
    }

    fn d_for_each_terminal_of_net<F>(&self, net: &Self::NetId, mut f: F)
        where F: FnMut(TerminalId<Self::D>) -> () {
        todo!()
    }

    fn d_each_terminal_of_net_vec(&self, net: &Self::NetId) -> Vec<TerminalId<Self::D>> {
        todo!()
    }

    fn d_each_terminal_of_net<'a>(&'a self, net: &Self::NetId)
                                  -> Box<dyn Iterator<Item=TerminalId<Self::D>> + 'a> {
        todo!()
    }
}