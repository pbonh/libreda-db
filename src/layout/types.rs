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

use super::index::{Index, IndexGenerator};
use super::cell::Cell;
use super::cell_instance::CellInstance;
use super::layout::LayerInfo;

pub type UInt = u32;
pub type SInt = i32;

/// Integer coordinate type.
pub type Coord = i32;

/// Data type used for identifying a layer.
pub type LayerIndex = Index<LayerInfo>;
pub(crate) type LayerIndexGenerator = IndexGenerator<LayerInfo>;

/// Data type used for identifying a cell.
pub type CellIndex = Index<Cell<Coord>>;
pub(crate) type CellIndexGenerator = IndexGenerator<Cell<Coord>>;

/// Data type used for identifying a cell instance.
pub type CellInstId = Index<CellInstance<Coord>>;
pub(crate) type CellInstIndexGenerator = IndexGenerator<CellInstance<Coord>>;

