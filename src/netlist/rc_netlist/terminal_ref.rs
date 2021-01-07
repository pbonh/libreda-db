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
//! `TerminalRef` can either be a `Pin` or a `PinInstance`.
//! It is used in cases where terminals of a net need to be represented generally without
//! making a distinction between pins and pin instances.

use std::rc::Rc;
use crate::netlist::prelude::*;

/// Describes where a net is connected to.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum TerminalRef {
    /// Connection to a pin of the parent circuit.
    Pin(Rc<Pin>),
    /// Connection to a pin of a sub-circuit instance.
    PinInstance(Rc<PinInstance>),
}

impl TerminalRef {
    /// Get the net to which the terminal is connected to.
    pub fn net(&self) -> Option<Rc<Net>> {
        match self {
            TerminalRef::Pin(p) => p.internal_net(),
            TerminalRef::PinInstance(p) => p.net()
        }
    }

    /// Get the ID of the terminal (pin or pin instance).
    pub fn terminal_id(&self) -> usize {
        match self {
            TerminalRef::Pin(p) => p.id(),
            TerminalRef::PinInstance(p) => p.id()
        }
    }
}

impl From<Rc<Pin>> for TerminalRef {
    fn from(p: Rc<Pin>) -> Self {
        TerminalRef::Pin(p.clone())
    }
}

impl From<Rc<PinInstance>> for TerminalRef {
    fn from(p: Rc<PinInstance>) -> Self {
        TerminalRef::PinInstance(p.clone())
    }
}