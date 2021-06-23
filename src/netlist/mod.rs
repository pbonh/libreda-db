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

//! Data structures for representation of circuit-level netlists.
//!
//! A netlist represents the connections of electrical components (here called 'circuits')
//! such as standard-cells or macro blocks. Each of the circuits can be composed of instances of
//! other circuits (recursion is not allowed).
//! A circuit serves as a template for circuit instances. The circuit defines 'pins' which represent
//! the electrical connectors to the circuit. Pins can be connected electrically by 'nets' which represent
//! an electrical potential like a metal wire. Nets are local to a circuit.
//!
//! Netlists can be implemented in different ways. This module provides [`RcNetlist`] and [`HashMapNetlist`].
//! [`RcNetlist`] emulates an object oriented approach, where circuits, instances, pins, nets, etc., are
//! represented as a resource counted struct. This is convenient in certain cases but is a complicated
//! pattern in respect of mutability. [`HashMapNetlist`] is a simpler structure without any interior mutability
//! patterns.
//!
//! Some traits are used to create abstractions over the different netlist implementations:
//! * [`NetlistBase`] defines basic functions for accessing and traversing a netlist.
//! * [`NetlistEdit`] defines basic functions for building and modifying a netlist.
//!
//! [`RcNetlist`]: rc_netlist::rc_netlist::RcNetlist
//! [`HashMapNetlist`]: hashmap_netlist::HashMapNetlist
//! [`NetlistBase`]: traits::NetlistBase
//! [`NetlistEdit`]: traits::NetlistEdit

pub mod prelude;
pub mod io;
pub mod rc_netlist;
// pub mod hashmap_netlist;
pub mod direction;
pub mod traits;