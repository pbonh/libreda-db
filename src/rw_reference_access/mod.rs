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

//! # Experimental
//! Wrapper around the [`HierarchyBase`], [`NetlistBase`], [`LayoutBase`] and [`L2NBase`] traits which
//! provide object like access methods.
//! In contrast to [`crate::reference_access`] this wrapper takes ownership of the underlying data structure
//! and hence also allows write access.
//! # Examples
//!
//! ```
//! use libreda_db::prelude::*;
//! use libreda_db::rw_reference_access::*;
//!
//! // Create some netlist/layout.
//! let mut chip = RwRefAccess::new(Chip::new());
//! let top = chip.create_cell("TOP".into());
//! let sub = chip.create_cell("SUB".into());
//! let sub_inst1 = top.create_instance(&sub, Some("inst1".into()));
//!
//! // `top` can now be used like an object to navigate the cell hierarchy, layout and netlist.
//! for subcell in top.each_cell_instance() {
//!     println!("{} contains {:?} which is a {}", top.name(), subcell.name(), subcell.template().name());
//! }
//!
//! ```

pub mod hierarchy_reference_access;

pub use hierarchy_reference_access::*;
