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

//! ID of an arc (net segment). The arc is defined by two terminals (pin or pin instance).


use std::hash::Hash;
use super::prelude::*;

/// An arc represents the direct path from one pin to another.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ArcId<N: NetlistBase + ?Sized> {
    start: TerminalId<N>,
    end: TerminalId<N>
}
