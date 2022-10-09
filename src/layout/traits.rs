// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Traits for layout data types.

#![allow(unused_variables)]

use crate::layout::types::{LayerInfo, UInt};
use crate::prelude::PropertyValue;
use crate::prelude::{Geometry, HierarchyMultithread, Rect};
use crate::traits::{HierarchyBase, HierarchyEdit};
use iron_shapes::transform::SimpleTransform;
use iron_shapes::CoordinateType;
use num_traits::Num;
use std::hash::Hash;

/// Most basic trait of a layout.
///
/// This traits specifies methods for accessing the components of a layout.
pub trait LayoutBase: HierarchyBase {
    /// Number type used for coordinates and distances.
    type Coord: CoordinateType + std::fmt::Debug + std::fmt::Display + Hash + 'static + Send + Sync;
    /// Number type for areas.
    /// This is possibly another type then `Coord` for the following reasons:
    /// * Distances and areas are semantically different.
    /// * In practice `i32` is a good choice for coordinates. However, computing areas in `i32` might
    /// easily lead to overflows. Hence a 64-bit integer type might be a better choice.
    type Area: Num + Copy + PartialOrd + From<Self::Coord> + 'static + Send + Sync;
    /// Layer identifier type.
    type LayerId: Eq + Hash + Clone + std::fmt::Debug + 'static;
    /// Shape identifier type.
    type ShapeId: Eq + Hash + Clone + std::fmt::Debug + 'static;

    /// Get the distance unit used in this layout in 'pixels per micron'.
    fn dbu(&self) -> Self::Coord;

    /// Iterate over all defined layers.
    fn each_layer(&self) -> Box<dyn Iterator<Item = Self::LayerId> + '_>;

    /// Get the `LayerInfo` data structure for this layer.
    fn layer_info(&self, layer: &Self::LayerId) -> LayerInfo<Self::NameType>;

    /// Find layer index by the (index, data type) tuple.
    fn find_layer(&self, index: UInt, datatype: UInt) -> Option<Self::LayerId>;

    /// Find layer index by the name.
    fn layer_by_name(&self, name: &str) -> Option<Self::LayerId>;

    /// Compute the bounding box of the shapes on one layer.
    /// The bounding box also includes all child cell instances.
    fn bounding_box_per_layer(
        &self,
        cell: &Self::CellId,
        layer: &Self::LayerId,
    ) -> Option<Rect<Self::Coord>>;

    /// Compute the bounding box of the cell over all layers.
    /// The bounding box is not defined if the cell is empty. In this
    /// case return `None`.
    fn bounding_box(&self, cell: &Self::CellId) -> Option<Rect<Self::Coord>> {
        self.each_layer()
            .map(|layer| self.bounding_box_per_layer(cell, &layer))
            .fold(None, |a, b| match (a, b) {
                (None, None) => None,
                (Some(a), None) | (None, Some(a)) => Some(a),
                (Some(a), Some(b)) => Some(a.add_rect(&b)),
            })
    }

    /// Iterate over the IDs of all shapes in the cell on a specific layer.
    fn each_shape_id(
        &self,
        cell: &Self::CellId,
        layer: &Self::LayerId,
    ) -> Box<dyn Iterator<Item = Self::ShapeId> + '_>;

    /// Call a function for each shape on this layer.
    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
    where
        F: FnMut(&Self::ShapeId, &Geometry<Self::Coord>) -> ();

    /// Access a shape by its ID.
    fn with_shape<F, R>(&self, shape_id: &Self::ShapeId, f: F) -> R
    where
        F: FnMut(&Self::LayerId, &Geometry<Self::Coord>) -> R;

    /// Get a clone of the shape geometry.
    fn shape_geometry(&self, shape_id: &Self::ShapeId) -> Geometry<Self::Coord> {
        self.with_shape(shape_id, |_, geo| geo.clone())
    }

    /// Get the layer of a shape.
    fn shape_layer(&self, shape_id: &Self::ShapeId) -> Self::LayerId {
        self.with_shape(shape_id, |layer, _| layer.clone())
    }

    /// Get the parent cell and the layer of a shape as a (cell, layer) tuple.
    fn parent_of_shape(&self, shape_id: &Self::ShapeId) -> (Self::CellId, Self::LayerId);

    /// Call a function `f` for each shape of this cell and its sub cells.
    /// Along to the geometric shape `f` also gets a transformation as argument.
    /// The transformation describes the actual position of the geometric shape relative to the `cell`.
    fn for_each_shape_recursive<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, mut f: F)
    where
        F: FnMut(SimpleTransform<Self::Coord>, &Self::ShapeId, &Geometry<Self::Coord>) -> (),
    {
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
            self.for_each_shape(&cell, layer, |id, g| f(tf.clone(), id, g));
        }
    }

    /// Get the geometric transform that describes the location of a cell instance relative to its parent.
    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<Self::Coord>;

    /// Get a property of a shape.
    fn get_shape_property(
        &self,
        shape: &Self::ShapeId,
        key: &Self::NameType,
    ) -> Option<PropertyValue> {
        None
    }
}

/// Additional requirement that all ID types are `Send + Sync` as needed for multithreading
pub trait LayoutMultithread: LayoutBase + HierarchyMultithread {}

impl<L> LayoutMultithread for L
where
    L: LayoutBase + HierarchyMultithread,
    L::LayerId: Send + Sync,
    L::ShapeId: Send + Sync,
    L::Coord: Send + Sync,
{
}

/// Access shapes and instances in a layout based on their locations.
pub trait RegionSearch: LayoutBase {
    /// Iterate over the IDs of all shapes (on all layers) whose bounding-box overlaps with the `search_region`.
    fn each_shape_in_region(
        &self,
        cell: &Self::CellId,
        search_region: &Rect<Self::Coord>,
    ) -> Box<dyn Iterator<Item = Self::ShapeId> + '_> {
        let cell = cell.clone(); // Get an owned ID.
        let search_region = search_region.clone(); // Get an owned rectangle.
        Box::new(self.each_layer().flat_map(move |layer_id| {
            self.each_shape_in_region_per_layer(&cell, &layer_id, &search_region)
        }))
    }

    /// Iterate over the IDs of all shapes (on a specific layer) whose bounding-box overlaps with the `search_region`.
    fn each_shape_in_region_per_layer(
        &self,
        cell: &Self::CellId,
        layer_id: &Self::LayerId,
        search_region: &Rect<Self::Coord>,
    ) -> Box<dyn Iterator<Item = Self::ShapeId> + '_>;

    /// Iterate over the IDs of all instances within the `cell` whose bounding-box overlaps with the `search_region`.
    fn each_cell_instance_in_region(
        &self,
        cell: &Self::CellId,
        search_region: &Rect<Self::Coord>,
    ) -> Box<dyn Iterator<Item = Self::CellInstId> + '_>;
}

/// Trait for layouts that support editing.
pub trait LayoutEdit: LayoutBase + HierarchyEdit {
    /// Set the distance unit used in this layout in 'pixels per micron'.
    fn set_dbu(&mut self, dbu: Self::Coord) {} // TODO: Remove default implementation.

    /// Create a new layer.
    /// Use `set_layer_name()` to define a name.
    fn create_layer(&mut self, index: UInt, datatype: UInt) -> Self::LayerId;

    /// Create a new layer with a specific ID. This is used to clone layer-stacks between layouts while preserving their IDs.
    /// Returns an `Err` when the ID already exists.
    fn create_layer_with_id(
        &mut self,
        layer_id: Self::LayerId,
        index: UInt,
        datatype: UInt,
    ) -> Result<(), ()>;

    /// Set the name of a layer or clear the layer name when passing `None`.
    /// This method should not change the ID of the layer.
    /// Returns the previous name of the layer.
    fn set_layer_name(
        &mut self,
        layer: &Self::LayerId,
        name: Option<Self::NameType>,
    ) -> Option<Self::NameType>;

    /// Insert a geometric shape into the parent cell.
    fn insert_shape(
        &mut self,
        parent_cell: &Self::CellId,
        layer: &Self::LayerId,
        geometry: Geometry<Self::Coord>,
    ) -> Self::ShapeId;

    /// Remove shape from the parent cell.
    fn remove_shape(&mut self, shape_id: &Self::ShapeId) -> Option<Geometry<Self::Coord>>;

    /// Replace the geometry of a shape.
    fn replace_shape(
        &mut self,
        shape_id: &Self::ShapeId,
        geometry: Geometry<Self::Coord>,
    ) -> Geometry<Self::Coord>;

    /// Set the geometric transform that describes the location of a cell instance relative to its parent.
    fn set_transform(&mut self, cell_inst: &Self::CellInstId, tf: SimpleTransform<Self::Coord>);

    /// Set a property of a shape.
    fn set_shape_property(
        &mut self,
        shape: &Self::ShapeId,
        key: Self::NameType,
        value: PropertyValue,
    ) {
    }
}
