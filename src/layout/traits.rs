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
use crate::layout::types::UInt;
use iron_shapes::transform::SimpleTransform;
use iron_shapes::CoordinateType;
use iron_shapes::shape::Geometry;

/// Most basic trait of a layout.
///
/// This traits specifies methods for accessing the components of a layout.
pub trait LayoutBase {
    /// Number type used for coordinates.
    type Coord: CoordinateType;
    /// Type for names of cells, instances, etc.
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
    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Get the name of the cell.
    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType;

    /// Get the name of the cell instance.
    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType>;

    /// Iterate over all child instance in a cell.
    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_>;

    /// Iterate over all cells that contain a child of type `cell`.
    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Iterate over all cells types that are instantiated in this `cell`.
    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Get the ID of the parent cell of this instance.
    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId;

    /// Get the ID of the template cell of this instance.
    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId;

    /// Find layer index by the (index, data type) tuple.
    fn find_layer(&self, index: UInt, datatype: UInt) -> Option<Self::LayerId>;

    // /// Iterate over all shapes on a layer.
    // fn each_shape(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=&Geometry<Self::Coord>> + '_>;

    /// Call a function for each shape on this layer.
    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(&Geometry<Self::Coord>) -> ();

    // /// Call a function for each shape on this layer.
    // fn for_each_shape_box<F>(&self, cell: &Self::CellId, layer: &Self::LayerId,
    //                          f: Box<dyn FnMut(&Geometry<Self::Coord>) -> ()>);
}


/// Trait for layouts that support editing.
pub trait LayoutEdit: LayoutBase {
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

    /// Insert a geometric shape into the parent cell.
    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>);
}
