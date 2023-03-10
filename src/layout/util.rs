// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Utility functions for dealing with layouts.

use crate::traits::{LayoutBase, LayoutEdit};
use iron_shapes::CoordinateType;
use std::borrow::Borrow;

/// Copy the shapes on a specific layer from one cell into another cell.
pub fn copy_shapes<LS, LT, C>(
    target_layout: &mut LT,
    target_cell: &LT::CellId,
    target_layer: &LT::LayerId,
    source_layout: &LS,
    source_cell: &LS::CellId,
    source_layer: &LS::LayerId,
) where
    C: CoordinateType,
    LS: LayoutBase<Coord = C>,
    LT: LayoutEdit<Coord = C>,
{
    source_layout.for_each_shape(source_cell, source_layer, |_, g| {
        let g2 = g.clone();
        target_layout.insert_shape(target_cell, target_layer, g2);
    })
}

/// Copy the shapes from all layers in a cell into another cell.
pub fn copy_shapes_all_layers<LS, LT, C>(
    target_layout: &mut LT,
    target_cell: &LT::CellId,
    source_layout: &LS,
    source_cell: &LS::CellId,
) where
    C: CoordinateType,
    LS: LayoutBase<Coord = C>,
    LT: LayoutEdit<Coord = C>,
{
    for source_layer in source_layout.each_layer() {
        // Find or create layer in target layout based on layer number or name.
        let layer_info = source_layout.layer_info(&source_layer);
        let target_layer =
            target_layout.find_or_create_layer(layer_info.index, layer_info.datatype);
        copy_shapes(
            target_layout,
            target_cell,
            &target_layer,
            source_layout,
            source_cell,
            &source_layer,
        )
    }
}

/// Copy all layers (without their contents) from a source layout into a destination layout.
/// # Panics
/// Panics if a layer number or layer name already exists.
pub fn copy_all_layers<LS, LT>(target_layout: &mut LT, source_layout: &LS)
where
    LS: LayoutBase,
    LT: LayoutEdit,
{
    for l in source_layout.each_layer() {
        copy_layer(target_layout, source_layout, &l);
    }
}

/// Copy all layers (without their contents) from a source layout into a destination layout and preserve
/// the layer IDs.
/// Source and target layout must have the same `LayerId` type.
///
/// # Panics
/// Panics if a layer number or layer name already exists.
pub fn copy_all_layers_preserve_ids<LS, LT>(target_layout: &mut LT, source_layout: &LS)
where
    LS: LayoutBase,
    LT: LayoutEdit<LayerId = LS::LayerId>,
{
    for l in source_layout.each_layer() {
        copy_layer_preserve_id(target_layout, source_layout, &l);
    }
}

/// Copy a layer (without its content) from a source layout into a destination layout.
///
/// # Panics
/// Panics if the layer number or layer name already exists.
pub fn copy_layer<LS, LT>(
    target_layout: &mut LT,
    source_layout: &LS,
    source_layer: &LS::LayerId,
) -> LT::LayerId
where
    LS: LayoutBase,
    LT: LayoutEdit,
{
    let layer_info = source_layout.layer_info(source_layer);
    let layer_id = target_layout.create_layer(layer_info.index, layer_info.datatype);

    // Convert between the name types via `&str`.
    let layer_name = layer_info.name.as_ref().map(|n| {
        let s: &str = n.borrow();
        s.to_string().into()
    });

    target_layout.set_layer_name(&layer_id, layer_name);

    layer_id
}

/// Copy a layer (without its content) from a source layout into a destination layout
/// while preserving the layer ID.
/// `LayerId` types of the both layouts must be the same.
///
/// # Panics
/// Panics if the layer number or layer name already exists.
pub fn copy_layer_preserve_id<LS, LT>(
    target_layout: &mut LT,
    source_layout: &LS,
    source_layer: &LS::LayerId,
) -> LT::LayerId
where
    LS: LayoutBase,
    LT: LayoutEdit<LayerId = LS::LayerId>,
{
    let layer_info = source_layout.layer_info(source_layer);
    let layer_id = source_layer.clone();
    target_layout
        .create_layer_with_id(layer_id.clone(), layer_info.index, layer_info.datatype)
        .expect("layer already exists");

    // Convert between the name types via `&str`.
    let layer_name = layer_info.name.as_ref().map(|n| {
        let s: &str = n.borrow();
        s.to_string().into()
    });

    target_layout.set_layer_name(&layer_id, layer_name);

    layer_id
}

/// Helper functions for layouts.
///
/// This trait is automatically implemented for all types which implement [`LayoutEdit`].
pub trait LayoutEditUtil: LayoutEdit {
    /// Create a layer or return an existing one.
    fn find_or_create_layer(&mut self, index: u32, datatype: u32) -> Self::LayerId {
        self.find_layer(index, datatype)
            .unwrap_or_else(|| self.create_layer(index, datatype))
    }
}

impl<L: LayoutEdit> LayoutEditUtil for L {}
