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
//! A cell is a container for geometric shapes and cell instances.

use iron_shapes::prelude::*;
use super::prelude::*;
use super::shape_collection::{Shapes, Shape};

use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use genawaiter::rc::Gen;
use std::hash::{Hash, Hasher};
use crate::property_storage::{PropertyStore, WithProperties};
use std::borrow::Borrow;

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
    index: CellIndex,
    /// Child cells.
    cell_instances: RefCell<HashMap<CellInstId, Rc<CellInstance<C>>>>,
    cell_instance_index_generator: RefCell<CellInstIndexGenerator>,
    /// Mapping from layer indices to geometry data.
    shapes_map: RefCell<HashMap<LayerIndex, Rc<Shapes<C>>>>,

    /// All the instances of this cell.
    cell_references: RefCell<HashSet<Rc<CellInstance<C>>>>,
    /// Set of cells that are dependencies of this cell.
    /// Stored together with a weak reference and a counter of how many instances of the dependency are present.
    /// This are the cells towards the leaves in the dependency tree.
    dependencies: RefCell<HashMap<CellIndex, (Weak<Self>, usize)>>,
    /// Cells that use an instance of this cell.
    /// This are the cells towards the root in the dependency tree.
    dependent_cells: RefCell<HashMap<CellIndex, (Weak<Self>, usize)>>,
    /// Properties related to this cell.
    cell_properties: RefCell<PropertyStore<String>>,
    /// Properties related to the instances in this cell.
    /// Instance properties are stored here for lower overhead of cell instances.
    pub (super) instance_properties: RefCell<HashMap<CellInstId, PropertyStore<String>>>,
}

impl<C: CoordinateType> Eq for Cell<C> {}

impl<C: CoordinateType> PartialEq for Cell<C> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
        // TODO: Compare parent layouts somehow.
    }
}

impl<C: CoordinateType> Hash for Cell<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<C: CoordinateType> Cell<C> {
    /// Create a new and empty cell.
    pub(super) fn new(name: Option<String>, index: CellIndex) -> Self {
        Cell {
            name: RefCell::new(name),
            self_reference: RefCell::default(),
            // layout: Weak::new(),
            cell_instances: Default::default(),
            index: index,
            shapes_map: Default::default(),
            cell_instance_index_generator: Default::default(),
            cell_references: Default::default(),
            dependencies: Default::default(),
            dependent_cells: Default::default(),
            cell_properties: Default::default(),
            instance_properties: Default::default(),
        }
    }

    /// Get index of this cell.
    pub fn index(&self) -> CellIndex {
        self.index
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
    pub fn clear_instances(&self) -> () {
        let all_instances: Vec<_> = self.cell_instances.borrow().values().cloned().collect();
        for inst in all_instances {
            self.remove_cell_instance(&inst)
        }
    }

    /// Remove all shapes and instances from this cell.
    pub fn clear(&self) -> () {
        self.clear_shapes();
        self.clear_instances();
    }

    /// Remove all shapes from the given layer.
    pub fn clear_layer(&self, layer_index: LayerIndex) -> () {
        self.shapes_map.borrow_mut().remove(&layer_index);
    }

    /// Insert a child cell instance.
    pub fn create_instance(&self, template_cell: &Rc<Cell<C>>, transform: SimpleTransform<C>) -> Rc<CellInstance<C>> {
        {
            // Check that creating this cell instance does not create a cycle in the dependency graph.
            // There can be no recursive instances.
            let mut stack: Vec<Rc<Cell<C>>> = vec![self.self_reference().upgrade().unwrap()];
            while let Some(c) = stack.pop() {
                if c.eq(&template_cell) {
                    // The cell to be instantiated depends on the current cell.
                    // This would insert a loop into the dependency tree.
                    // TODO: Don't panic but return an `Err`.
                    panic!("Cannot create recursive instances.");
                }
                // Follow the dependent cells towards the root.
                c.dependent_cells.borrow().values()
                    .map(|(dep, _)| dep.upgrade().unwrap()) // By construction this references should always be defined.
                    .for_each(|dep| stack.push(dep));
            }
        }

        // Generate fresh instance index.
        let index = self.cell_instance_index_generator.borrow_mut().next();
        let cell_inst = CellInstance {
            id: index,
            parent_cell_id: self.index(),
            cell: Rc::downgrade(template_cell),
            parent_cell: self.self_reference.borrow().clone(),
            transform: RefCell::new(transform),
        };
        let rc_cell_inst = Rc::new(cell_inst);
        self.cell_instances.borrow_mut().insert(index, rc_cell_inst.clone());


        // Remember dependency.
        {
            let mut dependencies = self.dependencies.borrow_mut();
            dependencies.entry(template_cell.index())
                .and_modify(|(_, c)| *c += 1)
                .or_insert((Rc::downgrade(template_cell), 1)); // First entry: Save weak reference with counter = 1.
        }

        // Remember dependency.
        {
            let mut dependent = template_cell.dependent_cells.borrow_mut();
            dependent.entry(self.index())
                .and_modify(|(_, c)| *c += 1)
                .or_insert((self.self_reference(), 1));// First entry: Save weak reference with counter = 1.
        }

        // Create an entry in the template cell.
        let was_not_present = template_cell.cell_references.borrow_mut()
            .insert(rc_cell_inst.clone());
        debug_assert!(was_not_present, "Cell instance with this index already existed!");

        // Sanity checks.
        #[cfg(debug_assertions)] {
            debug_assert_eq!(self.num_references(), self.dependent_cells.borrow().values()
                .map(|(_, n)| n).sum(), "self.num_references() is not consistent with the number of dependent cell.");
            debug_assert_eq!(template_cell.num_references(), template_cell.dependent_cells.borrow().values()
                .map(|(_, n)| n).sum(), "cell.num_references() is not consistent with the number of dependent cells.");

            // Check that dependencies are consistent.
            let dependencies = self.dependencies.borrow()
                .values()
                .map(|(c, _)| c.upgrade().unwrap().index())
                .sorted().collect_vec();

            let dependencies_derived = self.each_inst()
                .map(|c| c.cell_id())
                .unique()
                .sorted()
                .collect_vec();

            debug_assert_eq!(dependencies, dependencies_derived);
        }

        rc_cell_inst
    }

    /// Get the number of cell instances that reference this cell.
    pub fn num_references(&self) -> usize {
        self.cell_references.borrow().len()
    }

    /// Iterate over all cell instances that reference this cell.
    pub fn each_reference(&self) -> impl Iterator<Item=Rc<CellInstance<C>>> + '_ {
        let generator = Gen::new(|co| async move {
            for e in self.cell_references.borrow().iter().cloned() {
                co.yield_(e).await;
            }
        });
        generator.into_iter()
    }

    /// Remove the given cell instance from this cell.
     /// # Panics
     /// Panics if the cell instance does not live in this cell.
     /// TODO: Return an Err and let the user decide how to handle the error.
    pub fn remove_cell_instance(&self, cell_instance: &Rc<CellInstance<C>>) -> () {
        assert!(cell_instance.parent_cell().ptr_eq(&self.self_reference()),
                "Cell instance does not live in this cell.");

        // Remove dependency.
        {
            let mut dependencies = self.dependencies.borrow_mut();
            let template_cell_id = cell_instance.cell_id();
            // Decrement counter.
            let (_, count) = dependencies.entry(template_cell_id)
                .or_insert((Weak::new(), 0));
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                dependencies.remove(&template_cell_id);
            }
        }

        // Remove dependency.
        {
            let template_cell = cell_instance.cell().upgrade().unwrap();

            let mut dependent = template_cell.dependent_cells.borrow_mut();

            // Decrement counter.
            let (_, count) = dependent.entry(self.index())
                .or_insert((Weak::new(), 0));
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                dependent.remove(&self.index());
            }
        }

        // Remove the cell instance.
        self.cell_instances.borrow_mut().remove(&cell_instance.id())
            .unwrap();

        // Remove entry in the template cell.
        let remove_successful = cell_instance.cell().upgrade().unwrap()
            .cell_references.borrow_mut()
            .remove(cell_instance);
        assert!(remove_successful, "Failed to remove cell instance from 'cell_references'.");

        // Sanity checks.
        #[cfg(debug_assertions)]
            {
                debug_assert_eq!(self.num_references(), self.dependent_cells.borrow().values()
                    .map(|(_, n)| n).sum());
                let instance_ref = cell_instance.cell().upgrade().unwrap();
                debug_assert_eq!(instance_ref.num_references(), instance_ref.dependent_cells.borrow().values()
                    .map(|(_, n)| n).sum());
            }
    }

    /// Get reference to this cell.
    pub fn self_reference(&self) -> Weak<Self> {
        self.self_reference.borrow().clone()
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
            for i in self.cell_instances.borrow().values().cloned() {
                co.yield_(i).await;
            }
        });
        generator.into_iter()
    }

    /// Get all cells (not instances) that are direct children of this cell.
    pub fn each_cell_dependency(&self) -> impl Iterator<Item=Rc<Cell<C>>> + '_ {
        let generator = Gen::new(|co| async move {
            for (dep, _counter) in self.dependencies.borrow().values() {
                co.yield_(dep.upgrade().unwrap()).await;
            }
        });
        generator.into_iter()
    }

    /// Get all cells that directly depend on this cell,
    /// i.e. have an instance of this cell as a direct child.
    pub fn each_dependent_cell(&self) -> impl Iterator<Item=Rc<Cell<C>>> + '_ {
        let generator = Gen::new(|co| async move {
            for (dep, _counter) in self.dependent_cells.borrow().values() {
                co.yield_(dep.upgrade().unwrap()).await;
            }
        });
        generator.into_iter()
    }

    /// Returns true if this cell does not contain any other cell instances.
    pub fn is_leaf(&self) -> bool {
        self.cell_instances.borrow().is_empty()
    }
}

impl<C: CoordinateType> TryBoundingBox<C> for Cell<C> {
    fn try_bounding_box(&self) -> Option<Rect<C>> {
        // TODO: also take child instances into account.

        // Find the bounding box of all bounding boxes.
        let shapes_bbox = self.shapes_map.borrow().values()
            .filter_map(|shapes| shapes.try_bounding_box())
            .fold1(|a, b| a.add_rect(&b));

        shapes_bbox
    }
}


impl<C: CoordinateType> WithProperties for Cell<C> {
    type Key = String;

    fn with_properties<F, R>(&self, f: F) -> R
        where F: FnOnce(Option<&PropertyStore<Self::Key>>) -> R {
        f(Some(&self.cell_properties.borrow()))
    }

    fn with_properties_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut PropertyStore<Self::Key>) -> R {
        f(&mut self.cell_properties.borrow_mut())
    }
}
