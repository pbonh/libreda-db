// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Traits that define how certain design rules are represented.
//!
//! TBD

use num_traits::Num;
use crate::prelude::Orientation2D;

/// Define essential types used for expressing design rules.
pub trait RuleBase {
    /// Type used as layer identifier.
    type LayerId: Eq + Clone;
}

/// Define essential types used for expressing design rules based on distance relations.
pub trait DistanceRuleBase: RuleBase {
    /// Type used to express distances.
    type Distance: Num + Copy + PartialOrd;
    /// Type used to express areas.
    type Area: Num + Copy + PartialOrd;
}

/// Minimum spacing rules between shapes on the same layer.
pub trait MinimumSpacing: DistanceRuleBase {

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
pub trait MinimumWidth: DistanceRuleBase {

    /// Minimal width of a shape with a certain length.
    fn min_width(&self,
                 layer: &Self::LayerId,
                 shape_length: Option<Self::Distance>) -> Option<Self::Distance>;
}

/// Preferred routing direction on metal layers.
pub trait PreferredRoutingDirection: RuleBase {

    /// Get the preferred routing direction on this metal layer.
    fn preferred_routing_direction(&self, layer: &Self::LayerId) -> Option<Orientation2D>;
}