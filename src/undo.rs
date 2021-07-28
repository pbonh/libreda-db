/*
 * Copyright (c) 2020-2021 Thomas Kramer.
 *
 * This file is part of LibrEDA
 * (see https://codeberg.org/libreda/arboreus-cts).
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

//! Wrapper around netlist, layout and L2N structures that allows undoing of operations.
//!
//!
//! This is work in progress.
//!
//! # Caveat
//! Undoing removal of some objects does not preserve the ID of the object.
//! For example if a cell is deleted this can be undone. The restored cell, pins, instances, etc.
//! will have the same properties but different IDs.

use crate::traits::*;
use std::hash::Hash;
use std::borrow::Borrow;
use crate::netlist::direction::Direction;
use crate::netlist::traits::TerminalId;

/// Undo operations on the netlist.
pub enum NetlistUndoOp<T: NetlistBase> {
    /// Undo an operation on the hierarchy.
    HierarchyOp(HierarchyUndoOp<T>),
    /// Undo creating a pin.
    CreatePin(T::PinId),
    /// Store the old pin name.
    RenamePin(T::PinId, T::NameType),
    /// Undo creating a net.
    CreateNet(T::NetId),
    /// Store the previous net of the pin.
    ConnectPin(T::PinId, Option<T::NetId>),
    /// Store the previous net of the pin instance.
    ConnectPinInstance(T::PinInstId, Option<T::NetId>),
    /// Store old name of the net.
    RenameNet(T::NetId, Option<T::NameType>),
    // /// Store parent, old name and connected terminals of a net.
    // RemoveNet(T::CellId, Option<T::NameType>, Vec<TerminalId<T>>)

}

impl<T: NetlistBase> From<HierarchyUndoOp<T>> for NetlistUndoOp<T> {
    fn from(op: HierarchyUndoOp<T>) -> Self {
        Self::HierarchyOp(op)
    }
}

enum LayoutUndoOp<T: NetlistBase> {
    HierarchyOp(HierarchyUndoOp<T>)
}

enum L2NUndoOp<T: NetlistBase> {
    HierarchyOp(HierarchyUndoOp<T>),
    NetlistOp(NetlistUndoOp<T>),
    LayoutOp(LayoutUndoOp<T>),
}

/// Undo operation for hierarchy operations.
pub enum HierarchyUndoOp<T: HierarchyBase> {
    /// Undo creating a cell.
    CreateCell(T::CellId),
    /// Undo creating a cell instance.
    CreateCellInstance(T::CellInstId),
    /// Holds the previous name of the cell.
    RenameCell(T::CellId, T::NameType),
    /// Holds the previous name of the cell instance.
    RenameCellInst(T::CellInstId, Option<T::NameType>),
}

/// Wrapper around netlist, layout and L2N structures that allows undoing of operations.
pub struct Undo<'a, T, U> {
    chip: &'a mut T,
    transactions: Vec<U>,
}

impl<'a, T: HierarchyEdit, U> Undo<'a, T, U> {
    /// Return the number of undoable transactions.
    pub fn num_transactions(&self) -> usize {
        self.transactions.len()
    }

    /// Clear the undo buffer and make changes permanent.
    pub fn flush(&mut self) {
        self.transactions.clear()
    }

    fn undo_hierarchy_op(&mut self, op: HierarchyUndoOp<T>) {
        match op {
            HierarchyUndoOp::CreateCell(c) =>
                self.chip.remove_cell(&c),
            HierarchyUndoOp::CreateCellInstance(c) =>
                self.chip.remove_cell_instance(&c),
            HierarchyUndoOp::RenameCell(c, n) =>
                self.chip.rename_cell(&c, n),
            HierarchyUndoOp::RenameCellInst(c, n) =>
                self.chip.rename_cell_instance(&c, n)
        }
    }
}

impl<'a, T: NetlistEdit> Undo<'a, T, NetlistUndoOp<T>> {
    fn undo_netlist(chip: &'a mut T) -> Self {
        Self {
            chip,
            transactions: vec![],
        }
    }


    fn undo_netlist_op(&mut self, op: NetlistUndoOp<T>) {
        match op {
            NetlistUndoOp::HierarchyOp(op) =>
                self.undo_hierarchy_op(op),
            NetlistUndoOp::CreatePin(p) =>
                self.chip.remove_pin(&p),
            NetlistUndoOp::RenamePin(p, n) =>
                {self.chip.rename_pin(&p, n);},
            NetlistUndoOp::CreateNet( n) =>
                self.chip.remove_net(&n),
            NetlistUndoOp::ConnectPin(p, n) =>
                {self.chip.connect_pin(&p, n);}
            NetlistUndoOp::ConnectPinInstance(p, n) =>
                {self.chip.connect_pin_instance(&p, n);}
            NetlistUndoOp::RenameNet(net, name) =>
                {self.chip.rename_net(&net, name);}
            
        }
    }

    /// Undo the latest transaction.
    /// Does nothing if there's no transaction left to be undone.
    pub fn undo(&mut self) {
        if let Some(op) = self.transactions.pop() {
            self.undo_netlist_op(op)
        }
    }
}

impl<'a, T: HierarchyEdit> Undo<'a, T, HierarchyUndoOp<T>> {
    fn undo_hierarchy(chip: &'a mut T) -> Self {
        Self {
            chip,
            transactions: vec![],
        }
    }

    /// Undo the latest transaction.
    /// Does nothing if there's no transaction left to be undone.
    pub fn undo(&mut self) {
        if let Some(op) = self.transactions.pop() {
            self.undo_hierarchy_op(op)
        }
    }


    /// Undoes all transactions.
    pub fn undo_all(&mut self) {
        while !self.transactions.is_empty() {
            self.undo();
        }
    }

}

impl<'a, T: HierarchyBase, U> HierarchyBase for Undo<'a, T, U> {
    // This is nothing but simple redirection.
    type NameType = T::NameType;
    type CellId = T::CellId;
    type CellInstId = T::CellInstId;

    fn new() -> Self {
        unimplemented!()
    }

    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId> where Self::NameType: Borrow<N> {
        self.chip.cell_by_name(name)
    }

    fn cell_instance_by_name<N: ?Sized + Eq + Hash>(&self, parent_cell: &Self::CellId, name: &N) -> Option<Self::CellInstId> where Self::NameType: Borrow<N> {
        self.chip.cell_instance_by_name(parent_cell, name)
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        self.chip.cell_name(cell)
    }

    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.chip.cell_instance_name(cell_inst)
    }

    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.chip.parent_cell(cell_instance)
    }

    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.chip.template_cell(cell_instance)
    }

    fn for_each_cell<F>(&self, f: F) where F: FnMut(Self::CellId) -> () {
        self.chip.for_each_cell(f)
    }

    fn for_each_cell_instance<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.chip.for_each_cell_instance(cell, f)
    }

    fn for_each_cell_dependency<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.chip.for_each_cell_dependency(cell, f)
    }

    fn for_each_dependent_cell<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.chip.for_each_dependent_cell(cell, f)
    }

    fn for_each_cell_reference<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.chip.for_each_cell_reference(cell, f)
    }

    fn num_child_instances(&self, cell: &Self::CellId) -> usize {
        self.chip.num_child_instances(cell)
    }

    fn num_cells(&self) -> usize {
        self.chip.num_cells()
    }
}

impl<'a, T: HierarchyEdit, U: From<HierarchyUndoOp<T>>> HierarchyEdit for Undo<'a, T, U> {
    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId {
        let id = self.chip.create_cell(name);
        self.transactions.push(HierarchyUndoOp::CreateCell(id.clone()).into());
        id
    }

    fn remove_cell(&mut self, _cell_id: &Self::CellId) {
        unimplemented!()
    }

    fn create_cell_instance(&mut self, parent_cell: &Self::CellId, template_cell: &Self::CellId, name: Option<Self::NameType>) -> Self::CellInstId {
        let id = self.chip.create_cell_instance(parent_cell, template_cell, name);
        self.transactions.push(HierarchyUndoOp::CreateCellInstance(id.clone()).into());
        id
    }

    fn remove_cell_instance(&mut self, _inst: &Self::CellInstId) {
        unimplemented!()
    }

    fn rename_cell_instance(&mut self, inst: &Self::CellInstId, new_name: Option<Self::NameType>) {
        let prev_name = self.cell_instance_name(inst);
        self.chip.rename_cell_instance(inst, new_name);
        self.transactions.push(HierarchyUndoOp::RenameCellInst(inst.clone(), prev_name).into());
    }

    fn rename_cell(&mut self, cell: &Self::CellId, new_name: Self::NameType) {
        let prev_name = self.cell_name(cell);
        self.chip.rename_cell(cell, new_name);
        self.transactions.push(HierarchyUndoOp::RenameCell(cell.clone(), prev_name).into());
    }
}

impl<'a, T: NetlistBase, U> NetlistBase for Undo<'a, T, U> {
    type PinId = T::PinId;
    type PinInstId = T::PinInstId;
    type NetId = T::NetId;

    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId {
        self.chip.template_pin(pin_instance)
    }

    fn pin_direction(&self, pin: &Self::PinId) -> Direction {
        self.chip.pin_direction(pin)
    }

    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType {
        self.chip.pin_name(pin)
    }

    fn pin_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::PinId> where Self::NameType: Borrow<N> {
        self.chip.pin_by_name(parent_circuit, name)
    }

    fn parent_cell_of_pin(&self, pin: &Self::PinId) -> Self::CellId {
        self.chip.parent_cell_of_pin(pin)
    }

    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CellInstId {
        self.chip.parent_of_pin_instance(pin_inst)
    }

    fn parent_cell_of_net(&self, net: &Self::NetId) -> Self::CellId {
        self.chip.parent_cell_of_net(net)
    }

    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.chip.net_of_pin(pin)
    }

    fn net_of_pin_instance(&self, pin_instance: &Self::PinInstId) -> Option<Self::NetId> {
        self.chip.net_of_pin_instance(pin_instance)
    }

    fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.chip.net_zero(parent_circuit)
    }

    fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.chip.net_one(parent_circuit)
    }

    fn net_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::NetId> where Self::NameType: Borrow<N> {
        self.chip.net_by_name(parent_circuit, name)
    }

    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType> {
        self.chip.net_name(net)
    }

    fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> () {
        self.chip.for_each_pin(circuit, f)
    }

    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CellInstId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.chip.for_each_pin_instance(circuit_inst, f)
    }

    fn for_each_internal_net<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::NetId) -> () {
        self.chip.for_each_internal_net(circuit, f)
    }

    fn num_pins(&self, circuit: &Self::CellId) -> usize {
        self.chip.num_pins(circuit)
    }

    fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> () {
        self.chip.for_each_pin_of_net(net, f)
    }

    fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.chip.for_each_pin_instance_of_net(net, f)
    }
}

impl<'a, T, U> NetlistEdit for Undo<'a, T, U>
    where T: NetlistEdit,
        U: From<NetlistUndoOp<T>> + From<HierarchyUndoOp<T>> {
    fn create_pin(&mut self, circuit: &Self::CellId, name: Self::NameType, direction: Direction) -> Self::PinId {
        let id = self.chip.create_pin(circuit, name, direction);
        self.transactions.push(NetlistUndoOp::CreatePin(id.clone()).into());
        id
    }

    fn remove_pin(&mut self, id: &Self::PinId) {
        unimplemented!()
    }

    fn rename_pin(&mut self, pin: &Self::PinId, new_name: Self::NameType) -> Self::NameType {
        let prev_name = self.chip.pin_name(pin);
        self.transactions.push(NetlistUndoOp::RenamePin(pin.clone(), prev_name).into());
        self.chip.rename_pin(pin, new_name)
    }

    fn create_net(&mut self, parent: &Self::CellId, name: Option<Self::NameType>) -> Self::NetId {
        let id = self.chip.create_net(parent, name);
        self.transactions.push(NetlistUndoOp::CreateNet(id.clone()).into());
        id
    }

    fn rename_net(&mut self, net_id: &Self::NetId, new_name: Option<Self::NameType>) -> Option<Self::NameType> {
        let old_name = self.chip.rename_net(net_id, new_name);
        self.transactions.push(NetlistUndoOp::RenameNet(net_id.clone(), old_name.clone()).into());
        old_name
    }

    fn remove_net(&mut self, net: &Self::NetId) {
        // let old_name = self.net_name(net);
        // let old_terminals = self.each_terminal_of_net_vec(net);
        // self.transactions.push(NetlistUndoOp::RemoveNet(old_name, old_terminals).into());
        // self.chip.remove_net(net);
        unimplemented!()
    }

    fn connect_pin(&mut self, pin: &Self::PinId, net: Option<Self::NetId>) -> Option<Self::NetId> {
        let prev_net = self.chip.connect_pin(pin, net);
        self.transactions.push(NetlistUndoOp::ConnectPin(pin.clone(), prev_net.clone()).into());
        prev_net
    }

    fn connect_pin_instance(&mut self, pin: &Self::PinInstId, net: Option<Self::NetId>) -> Option<Self::NetId> {
        let prev_net = self.chip.connect_pin_instance(pin, net);
        self.transactions.push(NetlistUndoOp::ConnectPinInstance(pin.clone(), prev_net.clone()).into());
        prev_net
    }
}

#[test]
fn test_undoing() {
    use crate::chip::Chip;
    let mut chip = Chip::new();
    let mut undo = Undo::undo_netlist(&mut chip);

    let top = undo.create_cell("TOP".into());
    let top_a = undo.create_pin(&top, "A".into(), Direction::Input);
    let sub = undo.create_cell("SUB".into());
    let sub_b = undo.create_pin(&sub, "B".into(), Direction::Input);
    let inst = undo.create_cell_instance(&top, &sub, Some("inst1".into()));

    // Test undo renaming.
    undo.rename_cell(&top, "NewName".into());
    undo.rename_cell_instance(&inst, None);
    undo.undo();
    undo.undo();
    assert!(undo.cell_by_name("TOP").is_some());
    assert!(undo.cell_instance_by_name(&top, "inst1").is_some());



    // Undo create_cell_instance.
    assert_eq!(undo.num_child_instances(&top), 1);
    undo.undo();
    assert_eq!(undo.num_child_instances(&top), 0);


    assert_eq!(undo.num_cells(), 2);
    // Undo create pins and cells.
    undo.undo();
    undo.undo();
    undo.undo();
    undo.undo();
    assert_eq!(undo.num_cells(), 0);
}