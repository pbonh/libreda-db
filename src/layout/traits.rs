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

#![allow(unused_variables)]

use std::hash::Hash;
use crate::layout::types::{UInt, LayerInfo};
use iron_shapes::transform::SimpleTransform;
use iron_shapes::CoordinateType;
use crate::prelude::{Geometry, Rect};
use crate::traits::{HierarchyBase, HierarchyEdit};
use crate::prelude::PropertyValue;
use std::borrow::Borrow;

/// Most basic trait of a layout.
///
/// This traits specifies methods for accessing the components of a layout.
pub trait LayoutBase: HierarchyBase {
    /// Number type used for coordinates.
    type Coord: CoordinateType;
    /// Layer identifier type.
    type LayerId: Eq + Hash + Clone + std::fmt::Debug;
    /// Shape identifier type.
    type ShapeId: Eq + Hash + Clone + std::fmt::Debug;


    /// Get the distance unit used in this layout in 'pixels per micron'.
    fn dbu(&self) -> Self::Coord;

    /// Iterate over all defined layers.
    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_>;

    /// Get the `LayerInfo` data structure for this layer.
    fn layer_info(&self, layer: &Self::LayerId) -> &LayerInfo<Self::NameType>;

    /// Find layer index by the (index, data type) tuple.
    fn find_layer(&self, index: UInt, datatype: UInt) -> Option<Self::LayerId>;

    /// Find layer index by the name.
    fn layer_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::LayerId>
        where Self::NameType: Borrow<N>;

    /// Compute the bounding box of the shapes on one layer.
    /// The bounding box also includes all child cell instances.
    fn bounding_box_per_layer(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Option<Rect<Self::Coord>>;

    /// Compute the bounding box of the cell over all layers.
    fn bounding_box(&self, cell: &Self::CellId) -> Option<Rect<Self::Coord>> {
        self.each_layer()
            .map(|layer| self.bounding_box_per_layer(cell, &layer))
            .fold(None, |a, b| match (a, b) {
                (None, None) => None,
                (Some(a), None) | (None, Some(a)) => Some(a),
                (Some(a), Some(b)) => Some(a.add_rect(&b))
            })
    }

    /// Iterate over the IDs of all shapes in the cell on a specific layer.
    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_>;

    /// Call a function for each shape on this layer.
    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(&Self::ShapeId, &Geometry<Self::Coord>) -> ();

    /// Access a shape by its ID.
    fn with_shape<F, R>(&self, shape_id: &Self::ShapeId, f: F) -> R
        where F: FnMut(&Geometry<Self::Coord>) -> R;

    /// Call a function `f` for each shape of this cell and its sub cells.
    /// Along to the geometric shape `f` also gets a transformation as argument.
    /// The transformation describes the actual position of the geometric shape relative to the `cell`.
    fn for_each_shape_recursive<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, mut f: F)
        where F: FnMut(SimpleTransform<Self::Coord>, &Self::ShapeId, &Geometry<Self::Coord>) -> () {

        // This recursive iteration through the cells is implemented iteratively.
        // A plain recursive implementation is more difficult to handle due to the type system.

        // Stack for resolved recursion.
        let mut stack = Vec::new();
        stack.push((cell.clone(), SimpleTransform::identity()));

        while let Some((cell, tf)) = stack.pop() {

            // Push child instances.
            self.for_each_cell_instance(&cell, |inst| {
                let template = self.template_cell(&inst);
                let transform = self.get_transform(&inst);
                let tf2 = transform.then(&tf);
                stack.push((template, tf2));
            });

            // Process shapes of this cell.
            self.for_each_shape(&cell, layer,|id, g| f(tf.clone(), id, g));
        }
    }

    /// Get the geometric transform that describes the location of a cell instance relative to its parent.
    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<Self::Coord>;

    /// Get a property of a shape.
    fn get_shape_property(&mut self, cell: &Self::ShapeId, key: &Self::NameType) -> Option<PropertyValue> {
        None
    }

}


/// Trait for layouts that support editing.
pub trait LayoutEdit: LayoutBase + HierarchyEdit {

    /// Set the distance unit used in this layout in 'pixels per micron'.
    fn set_dbu(&mut self, dbu: Self::Coord) {} // TODO: Remove default implementation.

    /// Create a layer or return an existing one.
    fn find_or_create_layer(&mut self, index: UInt, datatype: UInt) -> Self::LayerId {
        self.find_layer(index, datatype)
            .unwrap_or_else(|| self.create_layer(index, datatype))
    }

    /// Create a new layer.
    /// Use `set_layer_name()` to define a name.
    fn create_layer(&mut self, index: UInt, datatype: UInt) -> Self::LayerId;

    /// Set the name of a layer or clear the layer name when passing `None`.
    /// This method should not change the ID of the layer.
    /// Returns the previous name of the layer.
    fn set_layer_name(&mut self, layer: &Self::LayerId, name: Option<Self::NameType>) -> Option<Self::NameType>;

    /// Insert a geometric shape into the parent cell.
    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId;

    /// Remove shape from the parent cell.
    fn remove_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                    shape_id: &Self::ShapeId) -> Option<Geometry<Self::Coord>>;

    /// Replace the geometry of a shape.
    fn replace_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                     shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>) -> Geometry<Self::Coord>;

    /// Set the geometric transform that describes the location of a cell instance relative to its parent.
    fn set_transform(&mut self, cell_inst: &Self::CellInstId, tf: SimpleTransform<Self::Coord>);

    /// Set a property of a shape.
    fn set_shape_property(&mut self, shape: &Self::ShapeId, key: Self::NameType, value: PropertyValue) {}

}
