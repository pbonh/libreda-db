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

//! Traits for layout data types.


use std::borrow::Borrow;
use std::hash::Hash;
use crate::layout::types::{UInt, SInt};
use iron_shapes::transform::SimpleTransform;
use iron_shapes::CoordinateType;

/// Most basic trait of a layout.
pub trait LayoutBase {
    /// Number type used for coordinates.
    type Coord: CoordinateType;
    /// Type for names of circuits, instances, pins, etc.
    type NameType: Eq + Hash + From<String> + Clone + Borrow<String> + Borrow<str>;
    /// Layer identifier type.
    type LayerId: Eq + Hash + Clone;
    /// Cell/module identifier type.
    type CellId: Eq + Hash + Clone;
    /// Cell instance identifier type.
    type CellInstId: Eq + Hash + Clone;


    /// Create a new empty netlist.
    fn new() -> Self;

    /// Find a cell by its name.
    /// Return the cell with the given name. Returns `None` if the cell does not exist.
    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N>;

    /// Iterate over all cells.
    fn each_cell<'a>(&'a self) -> Box<dyn Iterator<Item=Self::CellId> + 'a>;

    /// Iterate over all child instance in a cell.
    fn each_cell_instance<'a>(&'a self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + 'a>;

    /// Iterate over all cells that contain a child of type `cell`.
    fn each_dependent_cell<'a>(&'a self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a>;

    /// Iterate over all cells types that are instantiated in this `cell`.
    fn each_cell_dependency<'a>(&'a self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a>;

    /// Get the ID of the parent cell of this instance.
    fn parent_cell(&self, circuit_instance: &Self::CellInstId) -> Self::CellId;

    /// Get the ID of the template cell of this instance.
    fn template_cell(&self, circuit_instance: &Self::CellInstId) -> Self::CellId;

    /// Find layer index by the (index, data type) tuple.
    fn find_layer(&self, index: UInt, datatype: UInt) -> Option<Self::LayerId>;
}


/// Trait for layouts that support editing.
pub trait LayoutEdit
    where Self: LayoutBase {
    /// Create a layer or return an existing one.
    fn find_or_create_layer(&mut self, index: UInt, datatype: UInt) -> Self::LayerId;

    /// Create a new and empty cell.
    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId;

    /// Delete the given cell if it exists.
    fn remove_cell(&mut self, cell_id: &Self::CellId);

    /// Create a new instance of `template_cell` in `parent_cell`.
    fn create_cell_instance(&mut self,
                            parent_cell: &Self::CellId,
                            template_cell: &Self::CellId,
                            name: Option<Self::NameType>,
                            transform: SimpleTransform<Self::Coord>) -> Self::CellInstId;

    /// Remove cell instance if it exists.
    fn remove_cell_instance(&mut self, id: &Self::CellInstId);
}