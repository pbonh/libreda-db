//! A port is a multi-pin input or output connection of a circuit.

use super::prelude::Pin;
use std::rc::{Rc, Weak};
use crate::netlist::circuit::Circuit;

#[derive(Clone, Debug)]
pub struct Port {
    parent_circuit: Weak<Circuit>,
    pins: Vec<Rc<Pin>>
}