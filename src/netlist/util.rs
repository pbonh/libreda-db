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

//! Utility functions for dealing with netlists.

use crate::traits::{NetlistBase, NetlistEdit};

/// Non-modifying utility functions for netlists.
/// Import the this trait to use the utility functions all types that implement the `NetlistBase` trait.
pub trait NetlistUtil: NetlistBase {

    /// Check if the net is either the constant LOW or HIGH.
    fn is_constant_net(&self, net: &Self::NetId) -> bool {
        let parent = self.parent_cell_of_net(net);
        net == &self.net_zero(&parent) || net == &self.net_one(&parent)
    }

}

impl<N: NetlistBase> NetlistUtil for N {}

/// Modifying utility functions for netlists.
/// Import the this trait to use the utility functions all types that implement the `NetlistBase` trait.
pub trait NetlistEditUtil: NetlistEdit {

}

impl<N: NetlistEdit> NetlistEditUtil for N {}