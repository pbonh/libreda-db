// Copyright (c) 2020-2022 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Generalization of pins and pin instances.

use super::prelude::*;
use std::hash::{Hash, Hasher};

/// A terminal is a generalization of pins and pin instances.
#[derive(Debug)]
pub enum TerminalId<N: NetlistBase + ?Sized> {
    /// Terminal is a pin.
    PinId(N::PinId),
    /// Terminal is a pin instance.
    PinInstId(N::PinInstId),
}

impl<N: NetlistBase + ?Sized> Hash for TerminalId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TerminalId::PinId(p) => p.hash(state),
            TerminalId::PinInstId(p) => p.hash(state),
        }
    }
}

impl<N: NetlistBase + ?Sized> Eq for TerminalId<N> {}

impl<N: NetlistBase + ?Sized> PartialEq for TerminalId<N> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PinId(p1), Self::PinId(p2)) => p1 == p2,
            (Self::PinInstId(p1), Self::PinInstId(p2)) => p1 == p2,
            (_, _) => false,
        }
    }
}

impl<N: NetlistBase + ?Sized> Clone for TerminalId<N>
where
    N::PinId: Clone,
    N::PinInstId: Clone,
{
    fn clone(&self) -> Self {
        match self {
            TerminalId::PinId(p) => Self::PinId(p.clone()),
            TerminalId::PinInstId(p) => Self::PinInstId(p.clone()),
        }
    }
}
