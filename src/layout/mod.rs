// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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
pub mod types;
pub mod util;
