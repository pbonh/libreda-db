/*
 * Copyright (c) 2020-2022 Thomas Kramer.
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

//! Generalization of pins and pin instances.

use std::hash::{Hash, Hasher};
use super::prelude::*;

/// A terminal is a generalization of pins and pin instances.
#[derive(Debug)]
pub enum TerminalId<N: NetlistBase + ?Sized> {
    /// Terminal is a pin.
    PinId(N::PinId),
    /// Terminal is a pin instance.
    PinInstId(N::PinInstId),
}

impl<N: NetlistBase> Hash for TerminalId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            TerminalId::PinId(p) => p.hash(state),
            TerminalId::PinInstId(p) => p.hash(state)
        }
    }
}

impl<N: NetlistBase + ?Sized> Eq for TerminalId<N> {}

impl<N: NetlistBase + ?Sized> PartialEq for TerminalId<N> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PinId(p1), Self::PinId(p2)) => p1 == p2,
            (Self::PinInstId(p1), Self::PinInstId(p2)) => p1 == p2,
            (_, _) => false
        }
    }
}

impl<N: NetlistBase + ?Sized> Clone for TerminalId<N>
    where N::PinId: Clone, N::PinInstId: Clone {
    fn clone(&self) -> Self {
        match self {
            TerminalId::PinId(p) => Self::PinId(p.clone()),
            TerminalId::PinInstId(p) => Self::PinInstId(p.clone()),
        }
    }
}