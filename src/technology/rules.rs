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

//! Traits that define how certain design rules are represented.
//!
//! TBD

/// Define essential types used for expressing design rules.
pub trait RuleBase {
    /// Type used as layer identifier.
    type LayerId: Clone;
    /// Type used to express distances.
    type Distance: Copy + PartialOrd;
    /// Type used to express areas.
    type Area: Copy + PartialOrd;
}

/// Minimum spacing rules between shapes on the same layer.
pub trait MinimumSpacing: RuleBase {

    /// Absolute minimum spacing between two shapes on the `layer`.
    fn min_spacing_absolute(&self, layer: &Self::LayerId) -> Option<Self::Distance>;

    /// Minimum spacing between two shapes on the `layer` dependent on the geometries.
    fn min_spacing(&self,
                   layer: &Self::LayerId,
                   run_length: Self::Distance,
                   width: Self::Distance) -> Option<Self::Distance>;


    // Use another MinimumSpacing instance for same-net spacing.
    // fn min_spacing_same_net(layer: &Self::LayerId) -> Self::Distance;
}

/// Minimum with rules.
pub trait MinimumWidth: RuleBase {

    /// Minimal width of a shape with a certain length.
    fn min_width(&self,
                 layer: &Self::LayerId,
                 shape_length: Option<Self::Distance>) -> Option<Self::Distance>;
}
