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
//! Input and output interface definitions for netlists.

use std::io::{Read, Write};
use crate::netlist::traits::{NetlistEditTrait, NetlistTrait};

/// Read a netlist from a byte stream.
pub trait NetlistReader {
    /// Type of error that could happen while reading a netlist.
    type Error;

    /// Read a netlist from a byte stream and populate the netlist data structure.
    fn read_into_netlist<R: Read, N: NetlistEditTrait>(&self, reader: &mut R, netlist: &mut N) -> Result<(), Self::Error>;

    /// Read a netlist from a byte stream.
    fn read_netlist<R: Read, N: NetlistEditTrait>(&self, reader: &mut R) -> Result<N, Self::Error> {
        let mut netlist = N::new();
        self.read_into_netlist(reader, &mut netlist)?;
        Ok(netlist)
    }
}

/// Write a netlist to a byte stream.
pub trait NetlistWriter {
    /// Type of error that could happen while writing a netlist.
    type Error;

    /// Write the netlist data structure to a byte stream.
    fn write_netlist<W: Write, N: NetlistTrait>(&self, writer: &mut W, netlist: &N) -> Result<(), Self::Error>;
}
