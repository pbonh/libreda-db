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
//! A port is a multi-pin input or output connection of a circuit.

use super::prelude::{Circuit, Pin};
use std::rc::{Rc, Weak};

/// A port is a group of pins that are treated as a multi-wire signal.
#[derive(Clone, Debug)]
pub struct Port {
    parent_circuit: Weak<Circuit>,
    pins: Vec<Rc<Pin>>
}