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