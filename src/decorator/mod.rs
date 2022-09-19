// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Simplify the extension of netlists and layouts with the help of
//! wrappers and decorators.
//! 'Extending' classes and overriding methods as usual for OOP is not as easy in Rust.
//! This module contains helper traits which make such extension easier.
//!

pub mod hierarchy;
pub mod layout;
pub mod netlist;
pub mod l2n;

pub trait Decorator {
    /// The decorated type.
    type D;
    /// Get a reference to the underlying data structure.
    fn base(&self) -> &Self::D;
}

pub trait MutDecorator: Decorator {
    /// Get a mutable reference to the underlying data structure.
    fn mut_base(&mut self) -> &mut Self::D;
}