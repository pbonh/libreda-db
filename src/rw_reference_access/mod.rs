// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Experimental
//! Wrapper around the [`crate::traits::HierarchyBase`], [`crate::traits::NetlistBase`], [`crate::traits::LayoutBase`] and [`crate::traits::L2NBase`] traits which
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
