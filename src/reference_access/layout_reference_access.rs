// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::{LayoutBase, SimpleTransform, Rect, LayerInfo, Geometry};
use super::hierarchy_reference_access::*;

/// Trait that provides object-like read access to a layout structure and its elements.
pub trait LayoutReferenceAccess: LayoutBase
{
    /// Get a cell object by its ID.
    fn shape_ref(&self, shape_id: &Self::ShapeId) -> ShapeRef<'_, Self> {
        ShapeRef {
            base: self,
            id: shape_id.clone(),
        }
    }

    /// Get a layer object by its ID.
    fn layer_ref(&self, layer_id: &Self::LayerId) -> LayerRef<'_, Self> {
        LayerRef {
            base: self,
            id: layer_id.clone(),
        }
    }

    /// Iterate over all layers defined in this layout.
    fn each_layer_ref(&self) -> Box<dyn Iterator<Item=LayerRef<'_, Self>> + '_> {
        Box::new(
            self.each_layer()
                .map(move |id| LayerRef {
                    id,
                    base: self,
                })
        )
    }

    /// Get a layer object by the layer name.
    fn layer_ref_by_name(&self, name: &str) -> Option<LayerRef<'_, Self>> {
        self.layer_by_name(name)
            .map(|id| self.layer_ref(&id))
    }
}

impl<T: LayoutBase> LayoutReferenceAccess for T {}


impl<'a, L: LayoutBase> CellInstRef<'a, L> {
    /// Get the geometric transform that describes the location of a cell instance relative to its parent.
    pub fn get_transform(&self) -> SimpleTransform<L::Coord> {
        self.base().get_transform(&self.id)
    }
}

impl<'a, L: LayoutBase> CellRef<'a, L> {
    /// Iterate over all shapes on a layer.
    pub fn each_shape_per_layer(&self, layer_id: &L::LayerId) -> impl Iterator<Item=ShapeRef<L>> + '_ {
        self.base.each_shape_id(&self.id, layer_id)
            .map(move |id| ShapeRef {
                id,
                base: self.base,
            })
    }

    /// Iterate over all shapes defined in this cell.
    pub fn each_shape(&self) -> impl Iterator<Item=ShapeRef<L>> + '_ {
        self.base.each_layer()
            .flat_map(move |id| self.each_shape_per_layer(&id))
    }

    /// Get the bounding box of the shapes on a specific layer.
    pub fn bounding_box_per_layer(&self, layer_id: &L::LayerId) -> Option<Rect<L::Coord>> {
        self.base.bounding_box_per_layer(&self.id, layer_id)
    }

    /// Get the bounding box of the shapes on all layers.
    pub fn bounding_box(&self) -> Option<Rect<L::Coord>> {
        self.base.bounding_box(&self.id)
    }
}

/// Reference to a layer.
pub struct LayerRef<'a, L: LayoutBase + ?Sized> {
    /// Reference to the parent data structure.
    pub(super) base: &'a L,
    /// ID of the layer.
    pub(super) id: L::LayerId,
}

impl<'a, L: LayoutBase> LayerRef<'a, L> {
    /// Get the layer ID.
    pub fn id(&self) -> L::LayerId {
        self.id.clone()
    }

    /// Get the name of the layer.
    pub fn name(&self) -> Option<L::NameType> {
        self.layer_info().name.clone()
    }

    /// Get a reference to the layer-info structure.
    pub fn layer_info(&self) -> LayerInfo<L::NameType> {
        self.base.layer_info(&self.id)
    }
}

/// Reference to a shape.
pub struct ShapeRef<'a, L: LayoutBase + ?Sized> {
    /// Reference to the parent data structure.
    pub(super) base: &'a L,
    /// ID of the shape.
    pub(super) id: L::ShapeId,
}

impl<'a, L: LayoutBase> ShapeRef<'a, L> {
    /// Get the shape ID.
    pub fn id(&self) -> L::ShapeId {
        self.id.clone()
    }

    /// Get the cell where this shape lives.
    pub fn cell(&self) -> CellRef<L> {
        let id = self.base.parent_of_shape(&self.id).0;
        CellRef {
            id,
            base: self.base,
        }
    }

    /// Get the layer of the shape.
    pub fn layer(&self) -> LayerRef<L> {
        let id = self.base.parent_of_shape(&self.id).1;
        LayerRef {
            id,
            base: self.base,
        }
    }

    /// Get the cloned geometry struct representing this shape.
    pub fn geometry_cloned(&self) -> Geometry<L::Coord> {
        self.base.with_shape(&self.id,
                             |_layer, geo| {
                                 geo.clone()
                             })
    }
}