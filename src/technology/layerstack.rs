// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Definition of the metal layer stack as it is needed for routers.

use super::rules::RuleBase;

/// A routing layer is either a 'via' or 'routing/metal' layer.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RoutingLayerType {
    /// Via layer.
    Cut,
    /// Routing layer.
    Routing
}

/// Annotate a layer ID as 'via' or 'routing' layer.
pub struct RoutingLayer<LayerId> {
    /// ID of the layer. This identifies the layer in the data-base structures.
    id: LayerId,
    /// Type of the routing layer (via or metal).
    layer_type: RoutingLayerType
}

impl<LayerId> RoutingLayer<LayerId> {

    /// Associate a layer ID with a layer type.
    pub fn new(id: LayerId, layer_type: RoutingLayerType) -> Self {
        Self {
            id,
            layer_type
        }
    }

    /// Get the a reference to the ID of the layer.
    pub fn as_id(&self) -> &LayerId {
        &self.id
    }

    /// Get the ID of the layer.
    pub fn id(self) -> LayerId {
        self.id
    }

    /// Type of the layer.
    pub fn layer_type(&self) -> RoutingLayerType {
        self.layer_type
    }

    /// Check if layer is a via/cut layer.
    pub fn is_via_layer(&self) -> bool {
        match self.layer_type() {
            RoutingLayerType::Cut => true,
            RoutingLayerType::Routing => false,
        }
    }

    /// Check if layer is a metal layer.
    pub fn is_metal_layer(&self) -> bool {
        match self.layer_type() {
            RoutingLayerType::Cut => false,
            RoutingLayerType::Routing => true,
        }
    }
}

/// Define standardized access for routing and via layers.
pub trait RoutingLayerStack: RuleBase {

    /// Get the stack of routing and via layers in process order.
    fn layer_stack(&self) -> Vec<RoutingLayer<Self::LayerId>>;

    /// Get the stack of routing metal layers in process order.
    fn routing_layer_stack(&self) -> Vec<Self::LayerId> {
        self.layer_stack().into_iter()
            .filter(|l| l.is_metal_layer())
            .map(|l| l.id())
            .collect()
    }

    /// Get the stack of via layers in process order.
    fn via_layer_stack(&self) -> Vec<Self::LayerId> {
        self.layer_stack().into_iter()
            .filter(|l| l.is_via_layer())
            .map(|l| l.id())
            .collect()
    }

    /// Find the closest metal layer above the given layer.
    fn get_upper_metal_layer(&self, layer: &Self::LayerId) -> Option<Self::LayerId> {
        self.layer_stack().into_iter()
            .skip_while(|l| l.as_id() != layer)
            .skip(1)
            .find(|l| l.is_metal_layer())
            .map(|l| l.id())
    }

    /// Find the closest metal layer under the given layer.
    fn get_lower_metal_layer(&self, layer: &Self::LayerId) -> Option<Self::LayerId> {
        self.layer_stack().into_iter().rev()
            .skip_while(|l| l.as_id() != layer)
            .skip(1)
            .find(|l| l.is_metal_layer())
            .map(|l| l.id())
    }
}