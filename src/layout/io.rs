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

//! Input and output interface definitions for layouts.
//!
//! Implementations for the various layout formats are located in other crates.

use std::io::{Read, Write};
use super::layout::Layout;

/// Trait for reading a layout from a byte stream.
pub trait LayoutStreamReader {
    /// Type of error that could happen while reading a layout.
    type Error;
    /// Read a layout from a byte stream and populate the layout data structure.
    fn read_layout<R: Read>(&self, reader: &mut R, layout: &mut Layout) -> Result<(), Self::Error>;
}

/// Trait for writing a layout to a byte stream.
pub trait LayoutStreamWriter {
    /// Type of error that could happen while writing a layout.
    type Error;
    /// Write the layout data structure to a byte stream.
    fn write_layout<W: Write>(&self, writer: &mut W, layout: &Layout) -> Result<(), Self::Error>;
}