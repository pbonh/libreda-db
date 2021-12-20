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

//! *Experimental*: Wrapper around netlist, layout and L2N structures that allows undoing of operations.
//!
//! This is work in progress.
//! Missing things are:
//! * Undoing a operation on the hierarchy does not necessarily restore netlist, layout and l2n information.
//! * Undoing a netlist operation does not restore l2n information.
//! * Undoing a layout operation does not restore l2n information.
//! * Undoing does not restore user-defined properties.
//!
//! # Caveat
//! Undoing removal of some objects does not preserve the ID of the object.
//! For example if a cell is deleted this can be undone. The restored cell, pins, instances, etc.
//! will have the same properties but different IDs.

use crate::traits::*;
use crate::netlist::direction::Direction;
use crate::layout::prelude::{Rect, LayerInfo, Geometry, SimpleTransform};
use std::ops::Deref;
use crate::prelude::PropertyValue;
use crate::delegation::hierarchy::DelegateHierarchyBase;

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

/// Undo operation for `LayoutEdit` operations.
pub enum LayoutUndoOp<T: LayoutBase> {
    /// Undo an operation on the cell hierarchy.
    HierarchyOp(HierarchyUndoOp<T>),
    /// Store previous dbu.
    SetDbu(T::Coord),
    /// Store ID of the created layer.
    CreateLayer(T::LayerId),
    /// Store previous layer name.
    SetLayerName(T::LayerId, Option<T::NameType>),
    /// Store id of created shape.
    InsertShape(T::ShapeId),
    /// Store the geometry of the previous shape.
    RemoveShape {
        /// Parent cell of the removed shape.
        parent_cell: T::CellId,
        /// Layer of the removed shape.
        layer: T::LayerId,
        /// Geometry of the removed shape.
        geometry: Geometry<T::Coord>,
    },
    /// Store the old geometry of the shape.
    ReplaceShape(T::ShapeId, Geometry<T::Coord>),
    /// Store the old transform.
    SetTransform(T::CellInstId, SimpleTransform<T::Coord>),

}

impl<T: LayoutBase> From<HierarchyUndoOp<T>> for LayoutUndoOp<T> {
    fn from(op: HierarchyUndoOp<T>) -> Self {
        Self::HierarchyOp(op)
    }
}

/// Undo operation for `L2NEdit` operations.
#[allow(missing_docs)]
pub enum L2NUndoOp<T: L2NBase> {
    /// Undo an operation on the cell hierarchy.
    HierarchyOp(HierarchyUndoOp<T>),
    /// Undo a netlist operation.
    NetlistOp(NetlistUndoOp<T>),
    /// Undo a layout operation.
    LayoutOp(LayoutUndoOp<T>),
    /// Undo setting the net of a shape.
    SetNetOfShape {
        shape_id: T::ShapeId,
        previous_net: Option<T::NetId>,
    },
    /// Undo setting the pin of a shape.
    SetPinOfShape {
        shape_id: T::ShapeId,
        previous_pin: Option<T::PinId>,
    },
}

impl<T: L2NBase> From<HierarchyUndoOp<T>> for L2NUndoOp<T> {
    fn from(op: HierarchyUndoOp<T>) -> Self {
        Self::HierarchyOp(op)
    }
}

impl<T: L2NBase> From<NetlistUndoOp<T>> for L2NUndoOp<T> {
    fn from(op: NetlistUndoOp<T>) -> Self {
        Self::NetlistOp(op)
    }
}

impl<T: L2NBase> From<LayoutUndoOp<T>> for L2NUndoOp<T> {
    fn from(op: LayoutUndoOp<T>) -> Self {
        Self::LayoutOp(op)
    }
}

/// Undo operation for hierarchy operations.
pub enum HierarchyUndoOp<T: HierarchyBase> {
    /// Undo creating a cell.
    CreateCell(T::CellId),
    /// Undo creating a cell instance.
    CreateCellInstance(T::CellInstId),
    /// Holds the previous name of the cell.
    RenameCell {
        /// The renamed cell.
        cell: T::CellId,
        /// The name to be restored when undoing.
        previous_name: T::NameType,
    },
    /// Holds the previous name of the cell instance.
    RenameCellInst {
        /// The renamed instance.
        inst: T::CellInstId,
        /// The name to be restored when undoing.
        previous_name: Option<T::NameType>,
    },
}

/// Wrapper around netlist, layout and L2N structures that allows undoing of operations.
pub struct Undo<'a, T, U> {
    /// Underlying data structure.
    chip: &'a mut T,
    /// A list of performed transactions.
    /// To undo operations, the list has to be worked through from the end.
    transactions: Vec<U>,
}

impl<'a, T, U> Undo<'a, T, U> {
    /// Return the number of undoable transactions.
    pub fn num_transactions(&self) -> usize {
        self.transactions.len()
    }

    /// Clear the undo buffer and make changes permanent.
    pub fn flush(&mut self) {
        self.transactions.clear()
    }
}

impl<'a, T, U> Deref for Undo<'a, T, U> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.chip
    }
}

impl<'a, T: HierarchyEdit, U> Undo<'a, T, U> {
    /// Undo a hierarchy operation.
    fn undo_hierarchy_op(&mut self, op: HierarchyUndoOp<T>) {
        match op {
            HierarchyUndoOp::CreateCell(c) =>
                self.chip.remove_cell(&c),
            HierarchyUndoOp::CreateCellInstance(c) =>
                self.chip.remove_cell_instance(&c),
            HierarchyUndoOp::RenameCell { cell, previous_name } =>
                self.chip.rename_cell(&cell, previous_name),
            HierarchyUndoOp::RenameCellInst { inst, previous_name } =>
                self.chip.rename_cell_instance(&inst, previous_name)
        }
    }
}


impl<'a, T: L2NBase + 'static, U> L2NBase for Undo<'a, T, U> {
    fn shapes_of_net(&self, net_id: &Self::NetId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.chip.shapes_of_net(net_id)
    }

    fn shapes_of_pin(&self, pin_id: &Self::PinId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.chip.shapes_of_pin(pin_id)
    }

    fn get_net_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::NetId> {
        self.chip.get_net_of_shape(shape_id)
    }

    fn get_pin_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::PinId> {
        self.chip.get_pin_of_shape(shape_id)
    }
}

impl<'a, T: L2NEdit, U> Undo<'a, T, U> {
    /// Undo an operation on fused netlist and layout.
    fn undo_l2n_op(&mut self, op: L2NUndoOp<T>) {
        match op {
            // Redirect to base traits.
            L2NUndoOp::HierarchyOp(op) => self.undo_hierarchy_op(op),
            L2NUndoOp::NetlistOp(op) => self.undo_netlist_op(op),
            L2NUndoOp::LayoutOp(op) => self.undo_layout_op(op),
            // L2N specific operations
            L2NUndoOp::SetNetOfShape {
                shape_id,
                previous_net
            } => {
                self.chip.set_net_of_shape(&shape_id, previous_net);
            }
            L2NUndoOp::SetPinOfShape {
                shape_id,
                previous_pin
            } => {
                self.chip.set_pin_of_shape(&shape_id, previous_pin);
            }
        }
    }
}

impl<'a, T: L2NEdit> Undo<'a, T, L2NUndoOp<T>> {
    /// Create a wrapper around a fused layout and netlist which
    /// allows to undo operations.
    pub fn new_l2n_undo(chip: &'a mut T) -> Self {
        Self {
            chip,
            transactions: vec![],
        }
    }

    /// Undo the latest transaction.
    /// Does nothing if there's no transaction left to be undone.
    pub fn undo(&mut self) {
        if let Some(op) = self.transactions.pop() {
            self.undo_l2n_op(op)
        }
    }
}

impl<'a, T: LayoutEdit, U> Undo<'a, T, U> {
    /// Undo a layout operation
    fn undo_layout_op(&mut self, op: LayoutUndoOp<T>) {
        match op {
            LayoutUndoOp::HierarchyOp(op) => self.undo_hierarchy_op(op),
            LayoutUndoOp::SetDbu(dbu) => self.chip.set_dbu(dbu),
            LayoutUndoOp::CreateLayer(_id) => {
                // TODO
                log::error!("Creating a layer cannot be undone.");
            }
            LayoutUndoOp::SetLayerName(id, old_name) =>
                { self.chip.set_layer_name(&id, old_name); }
            LayoutUndoOp::InsertShape(id) =>
                { self.chip.remove_shape(&id); }
            LayoutUndoOp::RemoveShape { parent_cell, layer, geometry } => {
                self.chip.insert_shape(&parent_cell, &layer, geometry);
            }
            LayoutUndoOp::ReplaceShape(id, geometry) => {
                self.chip.replace_shape(&id, geometry);
            }
            LayoutUndoOp::SetTransform(inst, old_tf) => {
                self.chip.set_transform(&inst, old_tf)
            }
        }
    }
}

impl<'a, T: LayoutEdit> Undo<'a, T, LayoutUndoOp<T>> {
    /// Create a wrapper which allows to undo operations performed
    /// on the `LayoutEdit` trait.
    pub fn new_layout_undo(chip: &'a mut T) -> Self {
        Self {
            chip,
            transactions: vec![],
        }
    }

    /// Undo the latest transaction.
    /// Does nothing if there's no transaction left to be undone.
    pub fn undo(&mut self) {
        if let Some(op) = self.transactions.pop() {
            self.undo_layout_op(op)
        }
    }
}

impl<'a, T: NetlistEdit, U> Undo<'a, T, U> {
    /// Undo a netlist operation.
    fn undo_netlist_op(&mut self, op: NetlistUndoOp<T>) {
        match op {
            NetlistUndoOp::HierarchyOp(op) =>
                self.undo_hierarchy_op(op),
            NetlistUndoOp::CreatePin(p) =>
                self.chip.remove_pin(&p),
            NetlistUndoOp::RenamePin(p, n) =>
                { self.chip.rename_pin(&p, n); }
            NetlistUndoOp::CreateNet(n) =>
                self.chip.remove_net(&n),
            NetlistUndoOp::ConnectPin(p, n) =>
                { self.chip.connect_pin(&p, n); }
            NetlistUndoOp::ConnectPinInstance(p, n) =>
                { self.chip.connect_pin_instance(&p, n); }
            NetlistUndoOp::RenameNet(net, name) =>
                { self.chip.rename_net(&net, name); }
        }
    }
}

impl<'a, T: NetlistEdit> Undo<'a, T, NetlistUndoOp<T>> {
    /// Create a wrapper which allows to undo operations performed
    /// on the `NetlistEdit` trait.
    pub fn new_netlist_undo(chip: &'a mut T) -> Self {
        Self {
            chip,
            transactions: vec![],
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
    /// Create a wrapper which allows to undo operations performed
    /// on the `HierarchyEdit` trait.
    pub fn new_hierarchy_undo(chip: &'a mut T) -> Self {
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


impl<'a, H: HierarchyBase + 'static, U> DelegateHierarchyBase for Undo<'a, H, U> {
    type H = H;

    fn base(&self) -> &H {
        &self.chip
    }
}


impl<'a, T: HierarchyEdit + 'static, U: From<HierarchyUndoOp<T>>> HierarchyEdit for Undo<'a, T, U> {
    fn new() -> Self {
        unimplemented!()
    }

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
        let previous_name = self.d_cell_instance_name(inst);
        self.chip.rename_cell_instance(inst, new_name);
        self.transactions.push(HierarchyUndoOp::RenameCellInst { inst: inst.clone(), previous_name }.into());
    }

    fn rename_cell(&mut self, cell: &Self::CellId, new_name: Self::NameType) {
        let previous_name = self.d_cell_name(cell);
        self.chip.rename_cell(cell, new_name);
        self.transactions.push(HierarchyUndoOp::RenameCell { cell: cell.clone(), previous_name }.into());
    }
}

impl<'a, T: NetlistBase + 'static, U> NetlistBase for Undo<'a, T, U> {
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

    fn pin_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::PinId> {
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

    fn net_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::NetId> {
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
    where T: NetlistEdit + 'static,
          U: From<NetlistUndoOp<T>> + From<HierarchyUndoOp<T>> {
    fn create_pin(&mut self, circuit: &Self::CellId, name: Self::NameType, direction: Direction) -> Self::PinId {
        let id = self.chip.create_pin(circuit, name, direction);
        self.transactions.push(NetlistUndoOp::CreatePin(id.clone()).into());
        id
    }

    fn remove_pin(&mut self, _id: &Self::PinId) {
        unimplemented!("Removing a pin is not implemented to be undoable.")
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

    fn remove_net(&mut self, _net: &Self::NetId) {
        // let old_name = self.net_name(net);
        // let old_terminals = self.each_terminal_of_net_vec(net);
        // self.transactions.push(NetlistUndoOp::RemoveNet(old_name, old_terminals).into());
        // self.chip.remove_net(net);
        unimplemented!("Removing a net is not implemented to be undoable.")
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


impl<'a, T: LayoutBase + 'static, U> LayoutBase for Undo<'a, T, U> {

    // Pass-through all functions of the LayoutBase trait.

    type Coord = T::Coord;
    type LayerId = T::LayerId;
    type ShapeId = T::ShapeId;

    fn dbu(&self) -> Self::Coord {
        self.chip.dbu()
    }

    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_> {
        self.chip.each_layer()
    }

    fn layer_info(&self, layer: &Self::LayerId) -> LayerInfo<Self::NameType> {
        self.chip.layer_info(layer)
    }

    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.chip.find_layer(index, datatype)
    }

    fn layer_by_name(&self, name: &str) -> Option<Self::LayerId> {
        self.chip.layer_by_name(name)
    }

    fn bounding_box_per_layer(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Option<Rect<Self::Coord>> {
        self.chip.bounding_box_per_layer(cell, layer)
    }

    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.chip.each_shape_id(cell, layer)
    }

    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F) where F: FnMut(&Self::ShapeId, &Geometry<Self::Coord>) -> () {
        self.chip.for_each_shape(cell, layer, f)
    }

    fn with_shape<F, R>(&self, shape_id: &Self::ShapeId, f: F) -> R where F: FnMut(&Self::LayerId, &Geometry<Self::Coord>) -> R {
        self.chip.with_shape(shape_id, f)
    }

    fn parent_of_shape(&self, shape_id: &Self::ShapeId) -> (Self::CellId, Self::LayerId) {
        self.chip.parent_of_shape(shape_id)
    }

    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<Self::Coord> {
        self.chip.get_transform(cell_inst)
    }
}

impl<'a, T, U> LayoutEdit for Undo<'a, T, U>
    where T: LayoutEdit + 'static,
          U: From<LayoutUndoOp<T>> + From<HierarchyUndoOp<T>> {
    fn set_dbu(&mut self, dbu: Self::Coord) {
        self.transactions.push(LayoutUndoOp::SetDbu(self.dbu()).into());
        self.chip.set_dbu(dbu)
    }

    fn create_layer(&mut self, index: u32, datatype: u32) -> Self::LayerId {
        let id = self.chip.create_layer(index, datatype);
        self.transactions.push(LayoutUndoOp::CreateLayer(id.clone()).into());
        id
    }

    fn set_layer_name(&mut self, layer: &Self::LayerId, name: Option<Self::NameType>) -> Option<Self::NameType> {
        let old_name = self.layer_info(layer).name.clone();
        self.transactions.push(LayoutUndoOp::SetLayerName(layer.clone(), old_name).into());
        self.chip.set_layer_name(layer, name)
    }

    fn insert_shape(&mut self, parent_cell: &T::CellId, layer: &T::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId {
        let id = self.chip.insert_shape(parent_cell, layer, geometry);
        self.transactions.push(LayoutUndoOp::InsertShape(id.clone()).into());
        id
    }

    fn remove_shape(&mut self, shape_id: &Self::ShapeId) -> Option<Geometry<Self::Coord>> {
        let geometry = self.chip.remove_shape(shape_id);
        let (parent_cell, layer) = self.parent_of_shape(shape_id);
        if let Some(geometry) = &geometry {
            self.transactions.push(LayoutUndoOp::RemoveShape {
                parent_cell,
                layer,
                geometry: geometry.clone(),
            }.into());
        }
        geometry
    }

    fn replace_shape(&mut self, shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>) -> Geometry<Self::Coord> {
        let old_geometry = self.chip.replace_shape(shape_id, geometry);

        self.transactions.push(LayoutUndoOp::ReplaceShape(shape_id.clone(), old_geometry.clone()).into());

        old_geometry
    }

    fn set_transform(&mut self, cell_inst: &Self::CellInstId, tf: SimpleTransform<Self::Coord>) {
        let old_transform = self.get_transform(cell_inst);
        self.transactions.push(LayoutUndoOp::SetTransform(cell_inst.clone(), old_transform).into());
        self.chip.set_transform(cell_inst, tf)
    }

    fn set_shape_property(&mut self, shape: &Self::ShapeId, key: Self::NameType, _value: PropertyValue) {
        let _old_property = self.get_shape_property(shape, &key);
        unimplemented!("set_shape_property() is currently not undoable.")
    }
}

impl<'a, T, U> L2NEdit for Undo<'a, T, U>
    where T: L2NEdit + 'static,
          U: From<L2NUndoOp<T>> + From<LayoutUndoOp<T>> + From<NetlistUndoOp<T>> + From<HierarchyUndoOp<T>> {
    fn set_pin_of_shape(&mut self, shape_id: &Self::ShapeId, pin: Option<Self::PinId>) -> Option<Self::PinId> {
        let previous_pin = self.get_pin_of_shape(shape_id);
        self.transactions.push(L2NUndoOp::SetPinOfShape { shape_id: shape_id.clone(), previous_pin }.into());
        self.chip.set_pin_of_shape(shape_id, pin)
    }

    fn set_net_of_shape(&mut self, shape_id: &Self::ShapeId, net: Option<Self::NetId>) -> Option<Self::NetId> {
        let previous_net = self.get_net_of_shape(shape_id);
        self.transactions.push(L2NUndoOp::SetNetOfShape { shape_id: shape_id.clone(), previous_net }.into());
        self.chip.set_net_of_shape(shape_id, net)
    }
}


#[test]
fn test_hierarchy_undoing() {
    use crate::chip::Chip;
    let mut chip = Chip::new();
    let mut undo = Undo::new_netlist_undo(&mut chip);

    let top = undo.create_cell("TOP".into());
    let _top_a = undo.create_pin(&top, "A".into(), Direction::Input);
    let sub = undo.create_cell("SUB".into());
    let _sub_b = undo.create_pin(&sub, "B".into(), Direction::Input);
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