// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Data types used in the data base.

/// Default unsigned integer type.
pub type UInt = u32;
/// Default signed integer type.
pub type SInt = i32;

/// Integer coordinate type.
pub type Coord = i32;

/// Meta-data of a layer.
#[derive(Clone, Hash, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayerInfo<NameType> {
    /// Identifier of the layer.
    pub index: UInt,
    /// Identifier of the layer.
    pub datatype: UInt,
    /// Name of the layer.
    pub name: Option<NameType>,
}
