// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Traits for representation of circuit-level netlists.
//!
//! A netlist represents the connections of electrical components (here called 'circuits')
//! such as standard-cells or macro blocks. Each of the circuits can be composed of instances of
//! other circuits (recursion is not allowed).
//! A circuit serves as a template for circuit instances. The circuit defines 'pins' which represent
//! the electrical connectors to the circuit. Pins can be connected electrically by 'nets' which represent
//! an electrical potential like a metal wire. Nets are local to a circuit.
//!
//! The way a netlist can be accessed and modified is defined by the following two traits:
//! * [`NetlistBase`] defines basic functions for accessing and traversing a netlist.
//! * [`NetlistEdit`] defines basic functions for building and modifying a netlist.
//!
//! The [`Chip`] data structure implements the both traits.
//!
//! [`Chip`]: crate::chip::Chip
//! [`NetlistBase`]: traits::NetlistBase
//! [`NetlistEdit`]: traits::NetlistEdit

pub mod prelude;
pub mod io;
pub mod direction;
pub mod traits;
pub mod util;
pub mod terminal_id;
pub mod arc_id;