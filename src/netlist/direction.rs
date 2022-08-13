// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The type of a pin is specified by a signal direction.

/// Signal type for pins.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// Check if this direction is 'input'.
    pub fn is_input(&self) -> bool {
        self == &Direction::Input
    }
    /// Check if this direction is 'output'.
    pub fn is_output(&self) -> bool {
        self == &Direction::Output
    }

    /// Check if this direciton is either 'Supply' or 'Ground'.
    pub fn is_power(&self) -> bool {
        matches!(self, Direction::Supply | Direction::Ground)
    }
}