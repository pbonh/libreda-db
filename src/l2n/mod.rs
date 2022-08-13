// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Trait definitions for layouts fused with netlists.

pub mod util;

use super::traits::*;

/// Fused layout and netlist view.
/// This trait makes the link between netlist elements and layout elements.
pub trait L2NBase: LayoutBase + NetlistBase {
    /// Iterate over all shapes that are marked to belong to the specified net.
    fn shapes_of_net(&self, net_id: &Self::NetId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_>;
    /// Iterate over all shapes that are part of the pin.
    fn shapes_of_pin(&self, pin_id: &Self::PinId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_>;
    /// Get the net of a shape.
    fn get_net_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::NetId>;
    /// Get the pin that belongs to the shape (if any).
    fn get_pin_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::PinId>;
}

/// Additional requirement that all ID types are `Send + Sync` as needed for multithreading
pub trait L2NMultithread: LayoutMultithread + NetlistMultithread {}

impl<L> L2NMultithread for L
    where L: L2NBase + LayoutMultithread + NetlistMultithread
{}

/// Fused layout and netlist view.
/// This trait makes the link between netlist elements and layout elements.
pub trait L2NEdit: L2NBase + LayoutEdit + NetlistEdit {
    /// Create the link between a circuit pin and its shapes in the layout.
    /// Return the current pin.
    fn set_pin_of_shape(&mut self, shape_id: &Self::ShapeId, pin: Option<Self::PinId>) -> Option<Self::PinId>;
    /// Set the net of a shape.
    /// Return the current net.
    fn set_net_of_shape(&mut self, shape_id: &Self::ShapeId, net: Option<Self::NetId>) -> Option<Self::NetId>;
}