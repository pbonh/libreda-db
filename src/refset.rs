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
// use std::collections::HashSet;
// use std::cell::RefCell;
// use std::rc::Rc;
// use std::ops::Deref;
// use std::ptr;
// use std::hash::{Hash, Hasher};
// use std::collections::hash_map::RandomState;
//
// /// Wrapper around `Rc<RefCell<T>>` which additionally contains
// /// the `id` of the reference as it occurs in the parent `RefSet`.
// #[derive(Debug)]
// pub struct RcRef<T> {
//     reference: Rc<RefCell<T>>,
// }
//
// impl<T> Clone for RcRef<T> {
//     fn clone(&self) -> Self {
//         RcRef {
//             reference: self.reference.clone()
//         }
//     }
// }
//
// impl<T> Deref for RcRef<T> {
//     type Target = RefCell<T>;
//
//     fn deref(&self) -> &Self::Target {
//         self.reference.deref()
//     }
// }
//
// impl<T> Hash for RcRef<T> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         (Rc::as_ptr(&self.reference) as usize).hash(state)
//     }
// }
//
// impl<T> PartialEq for RcRef<T> {
//     fn eq(&self, other: &Self) -> bool {
//         ptr::eq(Rc::as_ptr(&self.reference), Rc::as_ptr(&other.reference))
//     }
// }
//
// impl<T> Eq for RcRef<T> {}
//
// /// Set-like container that provides interior mutability.
// #[derive(Clone, Debug)]
// pub struct RcSet<T, S = RandomState> {
//     container: HashSet<RcRef<T>, S>
// }
//
// impl<T> Default for RcSet<T> {
//     fn default() -> Self {
//         RcSet {
//             container: HashSet::new()
//         }
//     }
// }
//
//
// impl<T> RcSet<T> {
//     pub fn new() -> Self {
//         Self::default()
//     }
//
//     /// Delete all content, the ID generator is *not* reset.
//     pub fn clear(&mut self) -> () {
//         self.container.clear();
//     }
//
//     /// Insert an element and return a reference to it.
//     pub fn insert(&mut self, t: T) -> RcRef<T> {
//         let rcref = RcRef {
//             reference: Rc::new(RefCell::new(t))
//         };
//         self.container.insert(rcref.clone());
//         rcref
//     }
//
//     /// Remove an item from the set.
//     /// Return true if the item was successfully removed.
//     pub fn remove(&mut self, r: &RcRef<T>) -> bool {
//         self.container.remove(r)
//     }
//
//     /// Iterate over all elements in this container.
//     pub fn iter(&self) -> impl Iterator<Item=&Rc<RefCell<T>>> {
//         self.container.iter().map(|i| &i.reference)
//     }
//
//     pub fn into_iter(self) -> impl Iterator<Item=Rc<RefCell<T>>> {
//         self.container.into_iter().map(|i| i.reference)
//     }
//
//     /// Return number of elements in the container.
//     pub fn len(&self) -> usize {
//         self.container.len()
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.container.is_empty()
//     }
// }
//
// #[test]
// fn test_refset_insert() {
//     let mut s = RcSet::new();
//     let r1 = s.insert(42);
//     assert_eq!(r1.deref().clone().into_inner(), 42);
// }