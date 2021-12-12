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

//! Trait definitions for layouts fused with netlists.

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