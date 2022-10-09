// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Traits that define how certain design rules are represented.
//!
//! TBD

use crate::prelude::Orientation2D;
use num_traits::Num;

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
    fn min_spacing(
        &self,
        layer: &Self::LayerId,
        run_length: Self::Distance,
        width: Self::Distance,
    ) -> Option<Self::Distance>;

    // Use another MinimumSpacing instance for same-net spacing.
    // fn min_spacing_same_net(layer: &Self::LayerId) -> Self::Distance;
}

/// Minimum width rules.
pub trait MinimumWidth: DistanceRuleBase {
    /// Minimal width of a shape with a certain length.
    fn min_width(
        &self,
        layer: &Self::LayerId,
        shape_length: Option<Self::Distance>,
    ) -> Option<Self::Distance>;
}

/// Default width rules.
pub trait DefaultWidth: DistanceRuleBase {
    /// Default width of a wire segment of a certain length.
    fn default_width(
        &self,
        layer: &Self::LayerId,
        shape_length: Option<Self::Distance>,
    ) -> Option<Self::Distance>;
}

/// Preferred routing direction on metal layers.
pub trait PreferredRoutingDirection: RuleBase {
    /// Get the preferred routing direction on this metal layer.
    fn preferred_routing_direction(&self, layer: &Self::LayerId) -> Option<Orientation2D>;
}

/// Rules commonly used for routing.
pub trait RoutingRules:
    PreferredRoutingDirection + DefaultWidth + MinimumSpacing + MinimumWidth
{
    /// Get the default routing pitch on this layer for x and y directions.
    fn default_pitch(&self, layer: &Self::LayerId) -> Option<(Self::Distance, Self::Distance)>;

    /// Get the default routing pitch for wires with the preferred routing direction.
    /// Return `None` if no default pitch or no routing direction is defined for this layer.
    fn default_pitch_preferred_direction(&self, layer: &Self::LayerId) -> Option<Self::Distance> {
        let pitch = self.default_pitch(layer)?;
        let preferred_direction = self.preferred_routing_direction(layer)?;

        match preferred_direction {
            Orientation2D::Horizontal => Some(pitch.1),
            Orientation2D::Vertical => Some(pitch.0),
        }
    }
}
