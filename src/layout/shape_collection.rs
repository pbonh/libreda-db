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
use crate::prelude::*;
use super::index::{Index, IndexGenerator};

use itertools::Itertools;
use std::iter::FromIterator;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::collections::hash_map::Values;
use genawaiter;
use genawaiter::rc::Gen;

/// Wrapper around a `Geometry` struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape<T: CoordinateType> {
    index: Index<Self>,
    pub geometry: Geometry<T>,
}

impl<T: CoordinateType> Shape<T> {
    fn new<I: Into<Geometry<T>>>(index: Index<Shape<T>>, shape: I) -> Self {
        Shape {
            index,
            geometry: shape.into(),
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
    index_generator: RefCell<IndexGenerator<Shape<T>>>,
    shapes: RefCell<HashMap<Index<Shape<T>>, Rc<Shape<T>>>>,
}

impl<T: CoordinateType> Deref for Shape<T> {
    type Target = Geometry<T>;

    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl<T: CoordinateType> Shapes<T> {
    pub fn new() -> Self {
        Shapes {
            index_generator: Default::default(),
            shapes: Default::default(),
        }
    }

    /// Add a shape to the collection.
    pub fn insert<I: Into<Geometry<T>>>(&self, shape: I) -> Rc<Shape<T>> {
        let index = self.index_generator.borrow_mut().next();
        let shape = Rc::new(Shape::new(index, shape));
        self.shapes.borrow_mut().insert(index, Rc::clone(&shape));
        shape
    }

    /// Remove the shape from the collection if it exists.
    /// TODO: What if a shape from another `Shapes` collection is passed here?
    pub fn remove_shape(&self, shape: &Shape<T>) {
        self.shapes.borrow_mut().remove(&shape.index);
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

impl<'a, T: 'a + CoordinateType> FromIterator<&'a Geometry<T>> for Shapes<T> {
    fn from_iter<I: IntoIterator<Item=&'a Geometry<T>>>(iter: I) -> Self {
        let shapes = Shapes::new();
        for s in iter.into_iter() {
            shapes.insert(s.clone());
        }
        shapes
    }
}

impl<T: CoordinateType> FromIterator<Geometry<T>> for Shapes<T> {
    fn from_iter<I: IntoIterator<Item=Geometry<T>>>(iter: I) -> Self {
        let shapes = Shapes::new();
        for s in iter.into_iter() {
            shapes.insert(s);
        }
        shapes
    }
}

impl<T: CoordinateType> TryBoundingBox<T> for Shapes<T> {
    fn try_bounding_box(&self) -> Option<Rect<T>> {
        // TODO: Faster implementation by iterating over all points.
        self.with_shape_iter(|it| {
            it.filter_map(|s| s.try_bounding_box())
                .fold1(|a, b| a.add_rect(b))
        })
    }
}