//! Input and output interface definitions for layouts.

use std::io::{Read, Write};
use super::layout::Layout;

pub trait LayoutStreamReader {
    /// Type of error that could happen while reading a layout.
    type Error;
    /// Read a layout from a byte stream and populate the layout data structure.
    fn read_layout<R: Read>(&self, reader: &mut R, layout: &mut Layout) -> Result<(), Self::Error>;
}

pub trait LayoutStreamWriter {
    /// Type of error that could happen while writing a layout.
    type Error;
    /// Write the layout data structure to a byte stream.
    fn write_layout<W: Write>(&self, writer: &mut W, layout: &Layout) -> Result<(), Self::Error>;
}