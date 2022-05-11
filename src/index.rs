// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Data structure for creating indices related to some other type. This is used to create
//! handles for data stored in hash maps.

use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use num_traits::{Zero, One, PrimInt};

/// An identifier for an arbitrary type `T`.
/// This is a wrapper around `usize` which is bound to a type `T`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Index<T, Int = u32> {
    index: Int,
    phantom: PhantomData<T>,
}

impl<T, Int> fmt::Debug for Index<T, Int>
    where Int: std::fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Index({})", self.index)
    }
}

impl<T, Int: Hash> Hash for Index<T, Int> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl<T, Int: Copy> Copy for Index<T, Int> {}

impl<T, Int: Copy> Clone for Index<T, Int> {
    fn clone(&self) -> Self {
        Self::new(self.index)
    }
}

impl<T, Int: PartialEq> Eq for Index<T, Int> {}

impl<T, Int: PartialEq> PartialEq for Index<T, Int> {
    fn eq(&self, other: &Self) -> bool {
        self.index.eq(&other.index)
    }
}

impl<T, Int: PartialOrd + Ord> Ord for Index<T, Int> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T, Int: PartialOrd> PartialOrd for Index<T, Int> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T, Int: fmt::Display> fmt::Display for Index<T, Int> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T, Int: Copy> Index<T, Int> {
    pub(crate) fn new(index: Int) -> Self {
        Index {
            index,
            phantom: Default::default(),
        }
    }

    /// Get the integer value of this index.
    pub fn value(&self) -> Int {
        self.index
    }
}

/// Generator for incrementing `Index<T>` values.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct IndexGenerator<T, Int = u32> {
    counter: Int,
    phantom: PhantomData<T>,
}

impl<T, Int: PrimInt + Zero + One> Default for IndexGenerator<T, Int> {
    fn default() -> Self {
        Self::new(Int::zero())
    }
}

impl<T, Int: PrimInt + One> IndexGenerator<T, Int> {
    /// Create a new index generator.
    pub fn new(start: Int) -> Self {
        IndexGenerator {
            counter: start,
            phantom: Default::default(),
        }
    }

    // /// Jump forward with the counter.
    // /// # Panics
    // /// Panics when setting the counter to a smaller value.
    // pub fn set_counter(&mut self, value: usize) {
    //     assert!(value >= self.counter, "Cannot set the counter to a previous value.");
    //     self.counter = value;
    // }

    /// Generate a new index.
    pub fn next(&mut self) -> Index<T, Int> {
        let index = Index::new(self.counter);
        self.counter = self.counter + Int::one();
        index
    }
}
