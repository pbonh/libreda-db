/*
 * Copyright (c) 2020-2020 Thomas Kramer.
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

/// Signal type for pins.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    None,
    Input,
    Output,
    InOut,
    Clock,
    Supply,
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