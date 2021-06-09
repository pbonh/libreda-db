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

//! A shape collection represents a geometrical plane which contains geometrical shapes.

use crate::prelude::*;
use crate::index::{Index, IndexGenerator};

use itertools::Itertools;

use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::ops::Deref;
use std::collections::hash_map::Values;
use genawaiter;
use genawaiter::rc::Gen;
use crate::property_storage::{PropertyStore, WithProperties};
use std::hash::{Hash, Hasher};

/// Wrapper around a `Geometry` struct.
#[derive(Clone, Debug)]
pub struct Shape<T: CoordinateType> {
    /// Identifier of this shape.
    index: Index<Self>,
    /// The geometry of this shape.
    pub geometry: Geometry<T>,
    /// Weak reference to container.
    parent: Weak<Shapes<T>>,
}

impl<C: CoordinateType> Hash for Shape<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<C: CoordinateType> Eq for Shape<C> {}

impl<T: CoordinateType> PartialEq for Shape<T> {
    fn eq(&self, other: &Self) -> bool {
        // Compare by index and parent container.
        let eq = self.index == other.index &&
            self.parent.strong_count() > 0 && // Consider shapes without parent unequal.
            self.parent.ptr_eq(&other.parent);
        if eq {
            // Sanity check:
            debug_assert!(self.geometry == other.geometry,
                          "Geometry must be identical when the shape objects are equal."
            )
        }
        eq
    }
}

impl<T: CoordinateType> Shape<T> {
    fn new<I: Into<Geometry<T>>>(index: Index<Shape<T>>,
                                 shape: I,
                                 parent: Weak<Shapes<T>>) -> Self {
        Shape {
            index,
            geometry: shape.into(),
            parent,
        }
    }

    /// Get the index of this shape.
    pub fn index(&self) -> Index<Self> {
        self.index.clone()
    }
}

/// `Shapes<T>` is a collection of `Shape<T>` structs. Each of
/// the elements is assigned an index when inserted into the collection.
#[derive(Clone, Debug, Default)]
pub struct Shapes<T>
    where T: CoordinateType {
    /// Reference to this container itself.
    self_reference: RefCell<Weak<Self>>,
    /// Reference to the cell where this shape collection lives. Can be none.
    pub(super) parent_cell: Weak<Cell<T>>,
    index_generator: RefCell<IndexGenerator<Shape<T>>>,
    /// Shape elements.
    shapes: RefCell<HashMap<Index<Shape<T>>, Rc<Shape<T>>>>,
    /// Property stores for the shapes.
    shape_properties: RefCell<HashMap<Index<Shape<T>>, PropertyStore<String>>>,
}

impl<T: CoordinateType> Deref for Shape<T> {
    type Target = Geometry<T>;

    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl<T: CoordinateType> Shapes<T> {
    /// Create a new shapes object.
    /// It is not associated with any cell.
    pub fn new_rc() -> Rc<Self> {
        Self::new_rc_with_parent(Weak::default())
    }

    /// Create a new shapes object which is linked to the parent cell.
    pub(super) fn new_rc_with_parent(parent_cell: Weak<Cell<T>>) -> Rc<Self> {
        let shapes = Shapes {
            self_reference: Default::default(),
            parent_cell,
            index_generator: Default::default(),
            shapes: Default::default(),
            shape_properties: Default::default(),
        };

        let rc_shapes = Rc::new(shapes);
        // Store self-reference.
        *rc_shapes.self_reference.borrow_mut() = Rc::downgrade(&rc_shapes);

        rc_shapes
    }

    /// Create a new `Shapes` object and populate it with the geometries from the iterator.
    pub fn from_geometries<I: IntoIterator<Item=Geometry<T>>>(iter: I) -> Rc<Self> {
        let shapes = Shapes::new_rc();
        for s in iter.into_iter() {
            shapes.insert(s);
        }
        shapes
    }


    /// Add a shape to the collection.
    pub fn insert<I: Into<Geometry<T>>>(&self, shape: I) -> Rc<Shape<T>> {
        let index = self.index_generator.borrow_mut().next();
        let shape = Rc::new(Shape::new(index,
                                       shape,
                                       self.self_reference.borrow().clone()));
        self.shapes.borrow_mut().insert(index, Rc::clone(&shape));
        shape
    }

    /// Remove the shape from the collection if it exists. Return the removed shape.
    pub fn remove_shape(&self, shape_id: &Index<Shape<T>>) -> Option<Rc<Shape<T>>> {
        self.shapes.borrow_mut().remove(shape_id)
    }

    /// Return number of shapes in this container.
    pub fn len(&self) -> usize {
        self.shapes.borrow().len()
    }

    /// Tell if there are not shapes stored in this container.
    pub fn is_empty(&self) -> bool {
        self.shapes.borrow().is_empty()
    }

    /// Iterator over all shapes.
    pub fn each_shape(&self) -> impl Iterator<Item=Rc<Shape<T>>> + '_ {
        // Using a generator makes it possible to return an iterator over a value
        // borrowed from a `RefCell`.
        let generator = Gen::new(|co| async move {
            for s in self.shapes.borrow().values().cloned() {
                co.yield_(s).await;
            }
        });
        generator.into_iter()
    }

    /// Iterate over all shapes.
    pub fn with_shape_iter<F, R>(&self, f: F) -> R
        where F: FnOnce(Values<Index<Shape<T>>, Rc<Shape<T>>>) -> R,
    {
        f(self.shapes.borrow().values())
    }

    /// Call a closure on each shape.
    pub fn for_each_shape<F>(&self, f: F)
        where F: FnMut(&Rc<Shape<T>>),
    {
        self.shapes.borrow().values().for_each(f)
    }

    /// Get weak reference to the parent cell if there is any.
    pub fn parent_cell(&self) -> Weak<Cell<T>> {
        self.parent_cell.clone()
    }
}

// TODO
// impl<T: CoordinateType> IntoIterator for Shapes<T> {
//     type Item = Geometry<T>;
//     type IntoIter = std::vec::IntoIter<Self::Item>;
//
//     /// Iterator over all shapes.
//     fn into_iter(self) -> Self::IntoIter {
//         self.shapes.into_iter()
//     }
// }
//
// impl<'a, T: CoordinateType> IntoIterator for &'a Shapes<T> {
//     type Item = &'a Geometry<T>;
//     type IntoIter = std::slice::Iter<'a, Geometry<T>>;
//
//     /// Iterator over all shapes as references.
//     fn into_iter(self) -> Self::IntoIter {
//         (&self.shapes).into_iter()
//     }
// }

// impl<'a, T: 'a + CoordinateType> FromIterator<&'a Geometry<T>> for Shapes<T> {
//     fn from_iter<I: IntoIterator<Item=&'a Geometry<T>>>(iter: I) -> Self {
//         let shapes = Shapes::new();
//         for s in iter.into_iter() {
//             shapes.insert(s.clone());
//         }
//         shapes
//     }
// }
//
// impl<T: CoordinateType> FromIterator<Geometry<T>> for Shapes<T> {
//     fn from_iter<I: IntoIterator<Item=Geometry<T>>>(iter: I) -> Self {
//         let shapes = Shapes::new();
//         for s in iter.into_iter() {
//             shapes.insert(s);
//         }
//         shapes
//     }
// }

impl<T: CoordinateType> TryBoundingBox<T> for Shapes<T> {
    fn try_bounding_box(&self) -> Option<Rect<T>> {
        // TODO: Faster implementation by iterating over all points.
        self.with_shape_iter(|it| {
            it.filter_map(|s| s.try_bounding_box())
                .fold1(|a, b| a.add_rect(&b))
        })
    }
}

impl<C: CoordinateType> WithProperties for Shape<C> {
    type Key = String;

    fn with_properties<F, R>(&self, f: F) -> R
        where F: FnOnce(Option<&PropertyStore<Self::Key>>) -> R {
        f(
            // Get the property store from the parent cell.
            self.parent
                .upgrade()
                .unwrap()
                .shape_properties.borrow()
                .get(&self.index())
        )
    }

    fn with_properties_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut PropertyStore<Self::Key>) -> R {
        f(
            // Get the property store from the parent cell.
            self.parent
                .upgrade()
                .unwrap()
                .shape_properties.borrow_mut()
                .entry(self.index())
                .or_insert(PropertyStore::default())
        )
    }
}
