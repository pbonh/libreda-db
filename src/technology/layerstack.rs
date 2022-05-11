// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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
    type LayerId: Clone + Eq;

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