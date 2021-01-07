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

//! The type of a pin is specified by a signal direction.

/// Signal type for pins.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    /// No direction specified.
    None,
    /// Data input.
    Input,
    /// Data output.
    Output,
    /// Input and output.
    InOut,
    /// Clock input.
    Clock,
    /// Power VDD.
    Supply,
    /// Power ground.
    Ground,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::None
    }
}

impl Direction {
    /// Check if this direction.rs is 'input'.
    pub fn is_input(&self) -> bool {
        self == &Direction::Input
    }
    /// Check if this direction.rs is 'output'.
    pub fn is_output(&self) -> bool {
        self == &Direction::Output
    }
}