//! Input and output interface definitions for netlists.

use std::io::{Read, Write};
use super::netlist::Netlist;

pub trait NetlistReader {
    /// Type of error that could happen while reading a netlist.
    type Error;
    /// Read a netlist from a byte stream and populate the netlist data structure.
    fn read_netlist<R: Read>(&self, reader: &mut R, netlist: &mut Netlist) -> Result<(), Self::Error>;
}

pub trait NetlistWriter {
    /// Type of error that could happen while writing a netlist.
    type Error;
    /// Write the netlist data structure to a byte stream.
    fn write_netlist<W: Write>(&self, writer: &mut W, netlist: &Netlist) -> Result<(), Self::Error>;
}