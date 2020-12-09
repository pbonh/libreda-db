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
//! `iron-db` is a database for VLSI physical design. The core components are data structures for efficient
//! representation of geometries and circuit netlists for chip layouts.

pub extern crate iron_shapes;
extern crate itertools;
extern crate genawaiter;

// Public modules.
pub mod prelude;
pub mod netlist;
pub mod layout;

// Private modules.
// mod refset; // Not currently used.
// mod ref_wrapper; // Not currently used.
