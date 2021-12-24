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

//! Simplify the extension of netlists and layouts with the help of
//! wrappers and decorators.
//! 'Extending' classes and overriding methods as usual for OOP is not as easy in Rust.
//! This module contains helper traits which make such extension easier.
//!

pub mod hierarchy;
pub mod layout;

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