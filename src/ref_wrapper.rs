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
use std::collections::HashMap;
use std::cell::Ref;

/// Wrapper around `Ref<HashMap<K, V>>`.
/// This is used to make it possible to return an `IntoIterator` on a container
/// inside a `RefCell`.
pub struct HashMapRefWrapper<'a, K, V: 'a> {
    r: Ref<'a, HashMap<K, V>>
}

impl<'a, K, V: 'a> HashMapRefWrapper<'a, K, V> {
    pub fn new(r: Ref<'a, HashMap<K, V>>) -> Self {
        HashMapRefWrapper { r }
    }

    /// Iterate over the values of the hash map.
    pub fn values(&'a self) -> std::collections::hash_map::Values<'a, K, V> {
        self.r.values()
    }

    /// Iterate over the keys of the hash map.
    pub fn keys(&'a self) -> std::collections::hash_map::Keys<'a, K, V> {
        self.r.keys()
    }

    /// Return the size of the hash map.
    pub fn len(&'a self) -> usize {
        self.r.len()
    }

    /// Iterate over the key-value pairs of the hash map.
    pub fn iter(&'a self) -> std::collections::hash_map::Iter<'a, K, V> {
        self.r.iter()
    }
}

impl<'a, K, V: 'a + Clone> HashMapRefWrapper<'a, K, V> {
    /// Get the cloned values of the hash map in a `Vec`.
    pub fn values_vec(&'a self) -> Vec<V> {
        self.values().cloned().collect()
    }
}

impl<'a, 'b: 'a, K, V: 'a + Clone> IntoIterator for &'b HashMapRefWrapper<'a, K, V> {
    type Item = V;
    type IntoIter = std::iter::Cloned<std::collections::hash_map::Values<'a, K, V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.r.values().cloned()
    }
}

/// Wrapper around `Ref<_>`.
/// This is used to make it possible to return an `IntoIterator` on a container
/// inside a `RefCell`.
pub struct RefWrapper<'a, I> {
    r: Ref<'a, I>
}

impl<'a, I> RefWrapper<'a, I> {
    pub fn new(r: Ref<'a, I>) -> Self {
        RefWrapper { r }
    }
}

/// Implement `len()`.
impl<'a, I> RefWrapper<'a, I>
    where I: ExactSizeIterator {
    pub fn len(&'a self) -> usize {
        self.r.len()
    }
}

/// Implement IntoIterator.
impl<'a, 'b: 'a, I, T: 'a> IntoIterator for &'b RefWrapper<'a, I>
    where &'a I: IntoIterator<Item=T> {
    type Item = <&'a I as IntoIterator>::Item;
    type IntoIter = <&'a I as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.r.into_iter()
    }
}

impl<'a, 'b: 'a, I, T: 'a> RefWrapper<'a, I>
    where &'a I: IntoIterator<Item=T> {
    pub fn iter(&'b self) -> impl Iterator<Item=<&'a I as IntoIterator>::Item> {
        self.r.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use crate::ref_wrapper::RefWrapper;
    #[test]
    fn test_refwrapper_into_iter() {
        let vec = RefCell::new(vec![1, 2, 3]);
        let i = RefWrapper::new(vec.borrow());

        // assert_eq!(i.len(), 3);
        // into_iter()
        let collected: Vec<_> = i.into_iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_refwrapper_iter() {
        let vec = RefCell::new(vec![1, 2, 3]);
        let i = RefWrapper::new(vec.borrow());

        // iter()
        let collected: Vec<_> = i.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_refwrapper_iter_flatten() {
        let vec = RefCell::new(vec![vec![1, 2], vec![3]]);
        let i = RefWrapper::new(vec.borrow());

        // iter()
        let collected: Vec<_> = i.iter().flatten().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }
}

