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

//! Definition of the metal layer stack as it is needed for routers.

/// Annotate a layer ID as 'via' or 'routing' layer.
pub enum RoutingLayerType<LayerId> {
    /// Via layer.
    Cut(LayerId),
    /// Routing layer.
    Routing(LayerId)
}

/// Define standardized access for routing and via layers.
pub trait RoutingLayerStack {
    /// Identifier type for layers.
    type LayerId: Clone;

    /// Get the stack of routing and via layers in process order.
    fn layer_stack(&self) -> Vec<RoutingLayerType<Self::LayerId>>;

    /// Get the stack of routing metal layers in process order.
    fn routing_layer_stack(&self) -> Vec<Self::LayerId> {
        self.layer_stack().into_iter()
            .filter_map(|l| match l {
                RoutingLayerType::Routing(r) => Some(r),
                _ => None
            })
            .collect()
    }

    /// Get the stack of via layers in process order.
    fn via_layer_stack(&self) -> Vec<Self::LayerId> {
        self.layer_stack().into_iter()
            .filter_map(|l| match l {
                RoutingLayerType::Cut(c) => Some(c),
                _ => None
            })
            .collect()
    }
}