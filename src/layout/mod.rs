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

//! Geometrical layout data structures.
//!
//! A layout is a hierarchical structure. Its purpose is to efficiently represent the geometrical
//! properties of a silicon chip. Hence a layout consists of 'cells' which hold geometrical shapes such as polygons
//! on multiple layers. Cells also hold instances of other cells (recursion is not allowed though).
//! Typically a cells correspond to standard-cells or macros and will be instantiated possibly many times.
//! Each cell instance also holds the placement information, i.e. the location, rotation, mirroring and
//! possibly magnification.


pub mod prelude;

pub mod io;

pub mod traits;
pub mod algorithms;
pub mod types;
pub mod util;