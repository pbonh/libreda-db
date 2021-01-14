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

//! A layout data structure represents chip geometries. It consists of a hierarchical arrangement
//! of `Cell`s. Each cell contains geometric primitives that are grouped on `Layer`s.

use crate::prelude::*;
use std::collections::HashMap;
use crate::property_storage::PropertyStore;

/// Cell identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CellId(usize);

/// Cell instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CellInstId(usize);

/// Data structure which holds cells and cell instances.
///
/// # Examples
///
/// ```
/// use libreda_db::prelude::*;
/// let layout = Layout::new();
/// ```
#[derive(Default, Debug)]
pub struct Layout {
    /// Data-base unit. Pixels per micrometer.
    dbu: UInt,
    /// All cell templates.
    cells: HashMap<CellId, ()>,
    /// Counter for generating the next cell index.
    cell_index_generator: CellIndexGenerator,
    /// Lookup table for finding cells by name.
    cells_by_name: HashMap<RcString, CellIndex>,
    /// Counter for generating the next layer index.
    layer_index_generator: LayerIndexGenerator,
    /// Lookup table for finding layers by name.
    layers_by_name: HashMap<RcString, LayerIndex>,
    /// Lookup table for finding layers by index/datatype numbers.
    layers_by_index_datatype: HashMap<(UInt, UInt), LayerIndex>,
    /// Info structures for all layers.
    layer_info: HashMap<LayerIndex, LayerInfo>,
    /// Property storage for properties related to this layout.
    property_storage: PropertyStore<RcString>
}

/// Meta-data of a layer.
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct LayerInfo {
    /// Identifier of the layer.
    pub index: UInt,
    /// Identifier of the layer.
    pub datatype: UInt,
    /// Name of the layer.
    pub name: Option<RcString>,
}