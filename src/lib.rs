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

//! This crate is a database for VLSI physical design. The core components are traits that define
//! how netlist and layouts can be accessed and modified. Additionally the crate provides default
//! implementations of those traits for representation of chip layouts and netlists.
//!
//! ## Core parts
//!
//! An important part of this crate are trait definitions that describe the access methods
//! of cell hierarchies, netlists and layouts. The idea is that most algorithms should not be implemented
//! for concrete types but for this traits. For instance an algorithm that analyzes a netlist without looking at the layout
//! might be implemented for all types that implement the [`NetlistBase`] trait. In many
//! cases this allows to be agnostic of the actual netlist implementation. Hence the algorithm implementation
//! might be more portable.
//!
//! A fundamental idea is that all things (cells, instances, pins, nets, shapes, etc.) have unique
//! identifiers. The type of the identifiers is generically defined as an associated type of the traits.
//! In practice the identifiers might be for example integers but they can also some sort of smart pointer.
//!
//! The following are important traits which define how netlist and layouts can be accessed
//! and modified:
//! * [`HierarchyBase`] - traverse cell hierarchies
//! * [`HierarchyEdit`] - edit cell hierarchies
//! * [`NetlistBase`] - traverse netlists
//! * [`NetlistEdit`] - edit netlists
//! * [`LayoutBase`] - access layout shapes
//! * [`LayoutEdit`] - edit layout shapes
//! * [`L2NBase`] - access the links between layout shapes and netlist
//! * [`L2NEdit`] - edit the links between layout shapes and netlists
//!
//! Read more about netlists and layouts in the following modules:
//! * [`Netlist`]
//! * [`Layout`]
//!
//! The [`Chip`] struct implements the above traits and hence can be used as a default data base structure.
//!
//! ## Netlist/layout wrappers
//!
//! Additional functionality can be added to netlists and layout structures with the
//! following wrappers:
//!
//! * [`Undo`] - Make modifications reversible
//!
//! # Input/output
//! Reading and writing data base structures is generally left to other crates such as `libreda-oasis`,
//! `libreda-lefdef`, ...
//!
//! # Geometric primitives
//! Two dimensional geometrical primitives (polygons, rectangles, etc.) are re-exported from the [`iron_shapes`] crate.
//!
//! [`iron_shapes`]: iron_shapes
//! [`HierarchyBase`]: traits::HierarchyBase
//! [`HierarchyEdit`]: traits::HierarchyEdit
//! [`NetlistBase`]: netlist::traits::NetlistBase
//! [`NetlistEdit`]: netlist::traits::NetlistEdit
//! [`LayoutBase`]: netlist::traits::NetlistBase
//! [`LayoutEdit`]: netlist::traits::NetlistEdit
//! [`L2NBase`]: traits::L2NBase
//! [`L2NEdit`]: traits::L2NEdit
//! [`Netlist`]: netlist
//! [`Layout`]: layout
//! [`Chip`]: chip::Chip
//! [`Undo`]: undo

// Enforce documentation of the public API.
#![deny(missing_docs)]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// Re-exports: Crate for geometric primitives (points, polygons, ...).
pub use iron_shapes;

// Public modules.
pub mod prelude;
pub mod netlist;
pub mod layout;
pub mod index;
pub mod rc_string;
pub mod property_storage;
pub mod traits;
pub mod reference_access;
pub mod chip;
pub mod undo;
pub mod hierarchy_utils;
pub mod flat_view;


