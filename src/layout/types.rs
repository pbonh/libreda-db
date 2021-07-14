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

//! Data types used in the data base.

/// Default unsigned integer type.
pub type UInt = u32;
/// Default signed integer type.
pub type SInt = i32;

/// Integer coordinate type.
pub type Coord = i32;

/// Meta-data of a layer.
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct LayerInfo<NameType> {
    /// Identifier of the layer.
    pub index: UInt,
    /// Identifier of the layer.
    pub datatype: UInt,
    /// Name of the layer.
    pub name: Option<NameType>,
}