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
//! A cell is a container for geometric shapes and cell instances.

use crate::iron_shapes::prelude::*;
use super::prelude::*;
use super::shape_collection::{Shapes, Shape};

use itertools::Itertools;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use genawaiter::rc::Gen;

/// Mutable shared reference to a `Cell`.
pub type CellReference<C> = Rc<RefCell<Cell<C>>>;

/// A `Cell` is a container for geometrical shapes organized on different layers.
/// Additionally to the geometrical shapes a cell can also contain instances of other cells.
#[derive(Clone, Debug)]
pub struct Cell<C: CoordinateType> {
    /// Cell name.
    name: RefCell<Option<String>>,
    /// Reference to this cell itself.
    pub(super) self_reference: RefCell<Weak<Self>>,
    // /// The parent layout that holds this cell.
    // pub(crate) layout: Weak<Layout>,
    /// The index of this cell inside the layout. This is none if the cell does not belong to a layout.
    index: std::cell::Cell<CellIndex>,
    /// Child cells.
    instances: RefCell<HashMap<CellInstId, Rc<CellInstance<C>>>>,
    cell_instance_index_generator: RefCell<CellInstIndexdGenerator>,
    /// Mapping from layer indices to geometry data.
    shapes_map: RefCell<HashMap<LayerIndex, Rc<Shapes<C>>>>,
}

impl<C: CoordinateType> Cell<C> {
    /// Create a new and empty cell.
    pub(crate) fn new(name: Option<String>, index: CellIndex) -> Self {
        Cell {
            name: RefCell::new(name),
            self_reference: RefCell::default(),
            // layout: Weak::new(),
            instances: Default::default(),
            index: std::cell::Cell::new(index),
            shapes_map: Default::default(),
            cell_instance_index_generator: Default::default(),
        }
    }

    /// Return the cell name if it is defined.
    pub fn name(&self) -> Option<String> {
        self.name.borrow().clone()
    }

    /// Set a new cell name and return the old name.
    /// This does not update the lookup table in the layout object.
    pub(crate) fn set_name(&self, name: Option<String>) -> Option<String> {
        self.name.replace(name)
    }

    /// Remove all shapes from this cell.
    pub fn clear_shapes(&self) -> () {
        self.shapes_map.borrow_mut().clear();
    }

    /// Remove all instances from this cell.
    pub fn clear_insts(&self) -> () {
        self.instances.borrow_mut().clear();
    }

    /// Remove all shapes and instances from this cell.
    pub fn clear(&self) -> () {
        self.clear_shapes();
        self.clear_insts();
    }

    /// Remove all shapes from the given layer.
    pub fn clear_layer(&self, layer_index: LayerIndex) -> () {
        self.shapes_map.borrow_mut().remove(&layer_index);
    }

    /// Insert a child cell instance.
    /// TODO: Detect and deny recursion.
    /// TODO: Change to 'create_instance'.
    pub fn insert_instance(&self, cell_inst: CellInstance<C>) -> Rc<CellInstance<C>> {
        let index = self.cell_instance_index_generator.borrow_mut().next();
        let cell_inst = Rc::new(cell_inst);
        self.instances.borrow_mut().insert(index, cell_inst.clone());

        cell_inst
    }

    /// Get the shapes object for the given layer.
    pub fn shapes(&self, layer_index: LayerIndex) -> Option<Rc<Shapes<C>>> {
        self.shapes_map.borrow().get(&layer_index).cloned()
    }

    /// Get the mutable shapes object for the given layer or create a new one when no exists for this index.
    pub fn shapes_get_or_create(&self, layer_index: LayerIndex) -> Rc<Shapes<C>> {
        if let Some(shapes) = self.shapes(layer_index) {
            shapes
        } else {
            // Create a shapes object with a reference to this cell.
            let shapes = Shapes::new_rc_with_parent(self.self_reference.borrow().clone());
            // Associate the shape object with the layer index.
            self.shapes_map.borrow_mut().insert(layer_index, shapes.clone());
            shapes
        }
    }

    /// Return a `Vec` of all layers that contain at least one shape.
    pub fn each_used_layer(&self) -> Vec<LayerIndex> {
        self.shapes_map.borrow().iter()
            .filter(|(_idx, s)| s.len() > 0)
            .map(|(&i, _)| i)
            .collect()
    }

    /// Returns an iterator over all shapes of a given layer.
    pub fn each_shape(&self, layer_index: LayerIndex) -> impl Iterator<Item=Rc<Shape<C>>> + '_ {
        let generator = Gen::new(|co| async move {
            if let Some(shapes) = self.shapes(layer_index) {
                for s in shapes.each_shape() {
                    co.yield_(s).await;
                }
            };
        });
        generator.into_iter()
    }

    /// Returns an iterator over all child instances. (Returns reference to resource counted pointer).
    pub fn each_inst(&self) -> impl Iterator<Item=Rc<CellInstance<C>>> + '_ {
        // Using a generator makes it possible to return an iterator over a value
        // borrowed from a `RefCell`.
        let generator = Gen::new(|co| async move {
            for i in self.instances.borrow().values().cloned() {
                co.yield_(i).await;
            }
        });
        generator.into_iter()
    }

    /// Returns true if this cell does not contain any other cell instances.
    pub fn is_leaf(&self) -> bool {
        self.instances.borrow().is_empty()
    }
}

impl<C: CoordinateType> TryBoundingBox<C> for Cell<C> {
    fn try_bounding_box(&self) -> Option<Rect<C>> {
        // TODO: also take child instances into account.

        // Find the bounding box of all bounding boxes.
        let shapes_bbox = self.shapes_map.borrow().values()
            .filter_map(|shapes| shapes.try_bounding_box())
            .fold1(|a, b| a.add_rect(b));

        shapes_bbox
    }
}