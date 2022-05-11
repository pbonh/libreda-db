// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Input and output interface definitions for layouts.
//!
//! Implementations for the various layout formats are located in other crates.

use std::io::{Read, Write};
use crate::prelude::{LayoutBase, LayoutEdit};

/// Trait for reading a layout from a byte stream.
pub trait LayoutStreamReader {
    /// Type of error that could happen while reading a layout.
    type Error;
    /// Read a layout from a byte stream and populate the layout data structure.
    fn read_layout<R: Read, L: LayoutEdit<Coord=i32>>(&self, reader: &mut R, layout: &mut L) -> Result<(), Self::Error>;
}

/// Trait for writing a layout to a byte stream.
pub trait LayoutStreamWriter {
    /// Type of error that could happen while writing a layout.
    type Error;
    /// Write the layout data structure to a byte stream.
    fn write_layout<W: Write, L: LayoutBase<Coord=i32>>(&self, writer: &mut W, layout: &L) -> Result<(), Self::Error>;
}