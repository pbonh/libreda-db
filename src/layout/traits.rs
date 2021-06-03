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


use std::hash::Hash;
use crate::layout::types::{UInt, LayerInfo};
use iron_shapes::transform::SimpleTransform;
use iron_shapes::CoordinateType;
use iron_shapes::shape::Geometry;
use crate::traits::HierarchyBase;


/// Most basic trait of a layout.
///
/// This traits specifies methods for accessing the components of a layout.
pub trait LayoutBase: HierarchyBase {
    /// Number type used for coordinates.
    type Coord: CoordinateType;
    /// Layer identifier type.
    type LayerId: Eq + Hash + Clone;
    /// Shape identifier type.
    type ShapeId: Eq + Hash + Clone;


    /// Get the distance unit used in this layout in 'pixels per micron'.
    fn dbu(&self) -> Self::Coord;

    /// Iterate over all defined layers.
    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_>;

    /// Get the `LayerInfo` data structure for this layer.
    fn layer_info(&self, layer: &Self::LayerId) -> &LayerInfo;

    /// Find layer index by the (index, data type) tuple.
    fn find_layer(&self, index: UInt, datatype: UInt) -> Option<Self::LayerId>;

    /// Iterate over the IDs of all shapes in the cell on a specific layer.
    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_>;

    /// Call a function for each shape on this layer.
    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(&Geometry<Self::Coord>) -> ();

    // fn with_shapes<'a, F, R>(& self, cell: &Self::CellId, layer: &Self::LayerId, f: F) -> R
    //     where F: FnMut(dyn IntoIterator<Item=&'a Geometry<Self::Coord>>) -> R;

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
    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId;

    /// Remove shape from the parent cell.
    fn remove_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                    shape_id: &Self::ShapeId) -> Option<Geometry<Self::Coord>>;

    /// Replace the geometry of a shape.
    fn replace_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                     shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>) -> Option<Geometry<Self::Coord>>;
}
