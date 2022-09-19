// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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
//! * [`FlatView`] - Create an on-the-fly flattened view of a hierarchical structure.
//!
//! # Input/output
//! Reading and writing data base structures is generally left to other crates such as `libreda-oasis`,
//! `libreda-lefdef`, ...
//!
//! # Geometric primitives
//! Two dimensional geometrical primitives (polygons, rectangles, etc.) are re-exported from the [`iron_shapes`] crate.
//!
//! # Technology-related data
//! There is no complete standard way to deal with technology related data. A proposal on how to
//! also create an abstraction layer for technology data such as rules is here:
//!
//! * [`technology`] - interfaces for accessing technology related information such commonly used DRC rules
//!
//! [`iron_shapes`]: iron_shapes
//! [`HierarchyBase`]: traits::HierarchyBase
//! [`HierarchyEdit`]: traits::HierarchyEdit
//! [`NetlistBase`]: netlist::traits::NetlistBase
//! [`NetlistEdit`]: netlist::traits::NetlistEdit
//! [`LayoutBase`]: layout::traits::LayoutBase
//! [`LayoutEdit`]: layout::traits::LayoutEdit
//! [`L2NBase`]: traits::L2NBase
//! [`L2NEdit`]: traits::L2NEdit
//! [`Netlist`]: netlist
//! [`Layout`]: layout
//! [`Chip`]: chip::Chip
//! [`Undo`]: undo
//! [`FlatView`]: flat_view

// Enforce documentation of the public API.
#![deny(missing_docs)]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
extern crate num_traits;

/// Re-exports: Crate for geometric primitives (points, polygons, ...).
pub use iron_shapes;
pub use iron_shapes_booleanop;

// Public modules.
pub mod prelude;
pub mod traits;
pub mod hierarchy;
pub mod netlist;
pub mod layout;
pub mod l2n;
pub mod index;
pub mod rc_string;
pub mod property_storage;
pub mod reference_access;
pub mod rw_reference_access;
pub mod chip;
pub mod undo;
pub mod flat_view;
pub mod profile;
pub mod region_search;

pub mod technology;

mod decorator;
mod library;
mod slab_alloc;



