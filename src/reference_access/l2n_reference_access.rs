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

use crate::prelude::L2NBase;
use super::netlist_reference_access::*;
use super::layout_reference_access::*;

impl<'a, L: L2NBase> ShapeRef<'a, L> {

    /// Get the net which is connected to this shape, if any.
    pub fn net(&self) -> Option<NetRef<L>> {
        self.base.get_net_of_shape(&self.id)
            .map(|id| NetRef {
                id,
                base: self.base
            })
    }

    /// Get the pin which is connected to this shape, if any.
    pub fn pin(&self) -> Option<PinRef<L>> {
        self.base.get_pin_of_shape(&self.id)
            .map(|id| PinRef {
                id,
                base: self.base
            })
    }
}


impl<'a, L: L2NBase> NetRef<'a, L> {
    /// Iterate over all shapes attached to this net.
    pub fn each_shape(&self) -> impl Iterator<Item=ShapeRef<L>> {
        self.base.shapes_of_net(&self.id)
            .map(move |id| ShapeRef {
                id,
                base: self.base
            })
    }
}

impl<'a, L: L2NBase> PinRef<'a, L> {
    /// Iterate over all shapes attached to this pin.
    pub fn each_shape(&self) -> impl Iterator<Item=ShapeRef<L>> {
        self.base.shapes_of_pin(&self.id)
            .map(move |id| ShapeRef {
                id,
                base: self.base
            })
    }
}