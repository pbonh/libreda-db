// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Input and output interface definitions for netlists.
//!
//! Implementations for the various netlist formats are located in other crates.

use crate::netlist::traits::{NetlistBase, NetlistEdit};
use std::io::{Read, Write};

/// Read a netlist from a byte stream.
pub trait NetlistReader {
    /// Type of error that could happen while reading a netlist.
    type Error;

    /// Read a netlist from a byte stream and populate the netlist data structure.
    fn read_into_netlist<R: Read, N: NetlistEdit>(
        &self,
        reader: &mut R,
        netlist: &mut N,
    ) -> Result<(), Self::Error>;

    /// Read a netlist from a byte stream.
    fn read_netlist<R: Read, N: NetlistEdit>(&self, reader: &mut R) -> Result<N, Self::Error> {
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
    fn write_netlist<W: Write, N: NetlistBase>(
        &self,
        writer: &mut W,
        netlist: &N,
    ) -> Result<(), Self::Error>;
}
