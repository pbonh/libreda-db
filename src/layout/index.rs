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
//! Data structure for creating indices related to some other type. This is used to create
//! handles for data stored in hash maps.

use std::marker::PhantomData;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialOrd, Ord)]
pub struct Index<T> {
    index: usize,
    phantom: PhantomData<T>
}

impl<T> Hash for Index<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl<T: Clone> Copy for Index<T> {}

impl<T> Eq for Index<T> {}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index.eq(&other.index)
    }
}

impl<T> Index<T> {
    fn new(index: usize) -> Self {
        Index {
            index,
            phantom: Default::default()
        }
    }

    pub fn new_generator() -> IndexGenerator<T> {
        IndexGenerator::<T>::new()
    }
}

#[derive(Debug, Clone)]
pub struct IndexGenerator<T> {
    counter: usize,
    phantom: PhantomData<T>
}

impl<T> Default for IndexGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IndexGenerator<T> {
    /// Create a new index generator.
    pub fn new() -> Self {
        IndexGenerator {
            counter: 0,
            phantom: Default::default()
        }
    }

    /// Generate a new index.
    pub fn next(&mut self) -> Index<T> {
        let index = Index::new(self.counter);
        self.counter += 1;
        index
    }
}