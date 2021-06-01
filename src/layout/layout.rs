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
//! A layout data structure represents chip geometries. It consists of a hierarchical arrangement
//! of `Cell`s. Each cell contains geometric primitives that are grouped on `Layer`s.

use itertools::Itertools;

use genawaiter;
use genawaiter::rc::Gen;

use crate::prelude::*;
use super::errors::LayoutDbError;

use std::collections::HashMap;
use std::rc::Rc;
use std::hash::Hash;
use std::borrow::Borrow;
use crate::property_storage::{PropertyStore, WithProperties};
use std::cell::RefCell;
use crate::layout::traits::{LayoutBase, LayoutEdit};
use crate::layout::hashmap_layout::LayerId;
use crate::traits::HierarchyBase;


/// Data structure which holds cells and cell instances.
///
/// # Examples
///
/// ```rust
/// use libreda_db::prelude::*;
/// let mut layout = Layout::new();
/// ```
#[derive(Default, Debug)]
pub struct Layout {
    /// Data-base unit. Pixels per micrometer.
    pub dbu: UInt,
    /// All cell templates.
    cells: HashMap<CellIndex, Rc<Cell<Coord>>>,
    /// Counter for generating the next cell index.
    cell_index_generator: CellIndexGenerator,
    /// Lookup table for finding cells by name.
    cells_by_name: HashMap<String, CellIndex>,
    /// Counter for generating the next layer index.
    layer_index_generator: LayerIndexGenerator,
    /// Lookup table for finding layers by name.
    layers_by_name: HashMap<String, LayerIndex>,
    /// Lookup table for finding layers by index/datatype numbers.
    layers_by_index_datatype: HashMap<(UInt, UInt), LayerIndex>,
    /// Info structures for all layers.
    layer_info: HashMap<LayerIndex, LayerInfo>,
    /// Property storage for properties related to this layout.
    property_storage: RefCell<PropertyStore<String>>,
}

impl Layout {
    /// Create a new and empty layout.
    pub fn new() -> Self {
        let mut l = Layout::default();
        l.dbu = 1000;
        l
    }

    /// Create a new cell in this layout.
    /// Returns: Returns a handle to this cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut layout = Layout::new();
    /// // Create a cell and get it's index.
    /// let top_cell_index: CellIndex = layout.create_cell(Some("Top"));
    /// ```
    pub fn create_cell<S: Into<String>>(&mut self, cell_name: Option<S>) -> CellIndex {
        let cell_name = cell_name.map(|n| n.into());
        // Check for cell name collisions.
        if let Some(cell_name) = &cell_name {
            // TODO: what if cell name already exists?
            if let Some(_) = self.cell_by_name(cell_name.as_str()) {
                panic!("Cell with this name already exists.");
            }
        }

        // Create fresh cell index.
        let cell_index = self.cell_index_generator.next();

        let cell = Cell::new(cell_name.to_owned(), cell_index);
        let cell = Rc::new(cell);
        // Store reference to cell itself.
        *cell.self_reference.borrow_mut() = Rc::downgrade(&cell);

        self.cells.insert(cell_index, cell);
        if let Some(cell_name) = cell_name {
            self.cells_by_name.insert(cell_name, cell_index);
        }
        // Return reference to this cell.
        cell_index
    }


    /// Create a new cell in this layout.
    /// Returns: Returns a reference to this cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut layout = Layout::new();
    ///
    /// // Create a cell and directly get the index.
    /// let top_cell_ref = layout.create_and_get_cell(Some("Top"));
    /// ```
    pub fn create_and_get_cell<S: Into<String>>(&mut self, cell_name: Option<S>) -> Rc<Cell<Coord>> {
        let idx = self.create_cell(cell_name);
        self.cell_by_index(idx).unwrap() // This unwrap should succeed, otherwise there is a bug in this module.
    }

    /// Delete the given cell if it exists.
    pub fn remove_cell(&mut self, cell: &Rc<Cell<Coord>>) -> () {
        // Remove all circuit instances.
        let references = cell.each_reference().collect_vec();
        for inst in references {
            cell.remove_cell_instance(&inst)
        }
        // Check that now there are no references to this circuit anymore.
        debug_assert_eq!(cell.num_references(), 0);
        // Remove the circuit.
        if let Some(name) = cell.name() {
            self.cells_by_name.remove(&name).unwrap();
        }
        self.cells.remove(&cell.index());
    }

    /// Find a cell index by the cell name.
    /// Returns `None` if the cell name does not exist.
    #[inline(always)]
    pub fn cell_index_by_name<S: ?Sized>(&self, cell_name: &S) -> Option<CellIndex>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.cells_by_name.get(cell_name).copied()
    }

    /// Find a cell by its index.
    /// # Examples
    ///
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut layout = Layout::new();
    /// // Create a cell and get it's index.
    /// let top_cell_index: CellIndex = layout.create_cell(Some("Top"));
    /// // Get the reference to the cell by the index.
    /// let top_cell_ref = layout.cell_by_index(top_cell_index).unwrap();
    /// // Access the cell by the reference.
    /// assert_eq!(top_cell_ref.name().unwrap(), "Top");
    /// ```
    pub fn cell_by_index(&self, cell_index: CellIndex) -> Option<Rc<Cell<Coord>>> {
        self.cells.get(&cell_index).cloned()
    }

    /// Find a cell by its name.
    /// Returns `None` if there is no such cell.
    /// # Examples
    ///
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut layout = Layout::new();
    /// // Create a cell and get it's index.
    /// let top_cell_index: CellIndex = layout.create_cell(Some("Top"));
    /// // Get the reference to the cell by the index.
    /// let top_cell_ref = layout.cell_by_name("Top").unwrap();
    /// // Access the cell by the reference.
    /// assert_eq!(top_cell_ref.name().unwrap(), "Top");
    /// ```
    pub fn cell_by_name<S: ?Sized>(&self, cell_name: &S) -> Option<Rc<Cell<Coord>>>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.cell_index_by_name(cell_name)
            // This `unwrap` should not fail if the indices are kept consistent.
            .map(|i| self.cell_by_index(i).unwrap())
    }

    /// Change the name of a cell. The name is not allowed to already exist.
    /// Returns an error if the cell index is not found or the new name collides with an existing name.
    /// # Examples
    ///
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut layout = Layout::new();
    /// // Create a cell and get it's index.
    /// let a_cell_index: CellIndex = layout.create_cell(Some("A"));
    /// layout.rename_cell(a_cell_index, Some("B"));
    /// // Now a cell with name `A` does not exist anymore.
    /// assert!(layout.cell_by_name("A").is_none());
    /// // Get the reference to the cell by the index.
    /// let top_cell_ref = layout.cell_by_name("B").unwrap();
    /// // Access the cell by the reference.
    /// assert_eq!(top_cell_ref.name().unwrap(), "B");
    /// ```
    pub fn rename_cell<S: Into<String>>(&mut self, cell_index: CellIndex, new_name: Option<S>) -> Result<(), LayoutDbError> {
        let new_name = new_name.map(|n| n.into());

        let cell = self.cell_by_index(cell_index).ok_or(LayoutDbError::CellIndexNotFound)?;

        // Get old name.
        let old_name = cell.name();

        if new_name == old_name {
            // Nothing to change.
            return Ok(());
        }

        // Make sure the cell name does not already exist.
        if let Some(new_name) = &new_name {
            if self.cells_by_name.contains_key(new_name) {
                return Err(LayoutDbError::CellNameAlreadyExists(new_name.to_owned()));
            }
        }

        // Set new name.
        cell.set_name(new_name.to_owned());

        if let Some(old_name) = old_name {
            self.cells_by_name.remove(&old_name);
        }

        if let Some(new_name) = new_name {
            self.cells_by_name.insert(new_name, cell_index);
        }

        Ok(())
    }

    /// Find a cell by name or create it if it does not exist.
    pub fn get_or_create_cell_by_name(&mut self, cell_name: &str) -> CellIndex {
        match self.cell_index_by_name(cell_name) {
            Some(c) => c,
            None => self.create_cell(Some(cell_name))
        }
    }

    /// Get an iterator over all cells.
    pub fn each_cell(&self) -> impl Iterator<Item=&Rc<Cell<Coord>>> + ExactSizeIterator {
        self.cells.values().into_iter()
    }

    // /// Get an iterator over all cells.
    // pub fn each_cell_bottom_up(&self) -> impl Iterator<Item=&CellReference<Coord>> + ExactSizeIterator{
    //
    //     // fn walk_bottom_up(cell: &CellReference<Coord>) -> impl Iterator<Item=&CellReference<Coord>>  {
    //     //     RefCell::borrow(cell).each_inst_deref()
    //     //         .flat_map(|inst| walk_bottom_up(&inst.borrow().cell()))
    //     // }
    //     //
    //     // for c in self.each_cell() {
    //     //     RefCell::borrow(c).each_inst_deref();
    //     // }
    //
    //     unimplemented!();
    //     self.cells.values().into_iter()
    // }

    /// Returns true iff a cell with this name exists.
    pub fn has_cell<S: ?Sized>(&self, cell_name: &S) -> bool
        where String: Borrow<S>,
              S: Hash + Eq {
        self.cells_by_name.contains_key(cell_name)
    }

    /// Get the total number of cells in this layout.
    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    /// Find layer index by the name of the layer.
    pub fn find_layer_by_name<S: ?Sized>(&self, name: &S) -> Option<LayerIndex>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.layers_by_name.get(name).copied()
    }

    /// Find layer index by the (index, data type) tuple.
    pub fn find_layer(&self, index: UInt, datatype: UInt) -> Option<LayerIndex> {
        self.layers_by_index_datatype.get(&(index, datatype)).copied()
    }

    /// Find layer index by the (index, data type) tuple or create a new layer index if nothing can be found.
    pub fn find_or_create_layer(&mut self, index: UInt, datatype: UInt) -> LayerIndex {
        let layer = self.find_layer(index, datatype);
        if let Some(layer) = layer {
            layer
        } else {
            // Find next free layer index.
            let layer_index = self.layer_index_generator.next();
            // Create new entries in the layer lookup tables.
            self.layers_by_index_datatype.insert((index, datatype), layer_index);

            let info = LayerInfo { index, datatype, name: None };
            self.layer_info.insert(layer_index, info);
            layer_index
        }
    }

    /// Get the read-only layer info datastructure for the given layer.
    pub fn get_layer_info(&self, layer_index: LayerIndex) -> Option<&LayerInfo> {
        self.layer_info.get(&layer_index)
    }

    /// Get the mutable layer info datastructure for the given layer.
    pub fn get_layer_info_mut(&mut self, layer_index: LayerIndex) -> Option<&mut LayerInfo> {
        self.layer_info.get_mut(&layer_index)
    }

    /// Set the name of a layer. `None` indicates that the layer has no name.
    pub fn set_layer_name(&mut self, layer_index: LayerIndex, name: Option<String>) -> () {
        if let Some(i) = self.layer_info.get_mut(&layer_index) {
            i.name = name
        }
    }
}

impl WithProperties for Layout {
    type Key = String;

    fn with_properties<F, R>(&self, f: F) -> R
        where F: FnOnce(Option<&PropertyStore<Self::Key>>) -> R {
        f(Some(&self.property_storage.borrow()))
    }

    fn with_properties_mut<F, R>(&self, f: F) -> R where F: FnOnce(&mut PropertyStore<Self::Key>) -> R {
        f(&mut self.property_storage.borrow_mut())
    }
}

#[test]
fn test_layout_properties() {
    // Test setting and getting a property.

    let layout = Layout::default();

    layout.set_property("my_string_property".to_string(), "string_value".to_string());
    assert!(!layout.property_str("my_string_property").is_none());
}


impl HierarchyBase for Layout {
    type NameType = String;
    type CellId = Rc<Cell<Coord>>;
    type CellInstId = Rc<CellInstance<Coord>>;

    fn new() -> Self {
        Layout::new()
    }

    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId> where Self::NameType: Borrow<N> {
        self.cell_by_name(name)
    }

    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.each_cell().cloned())
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        cell.name().unwrap()
    }

    fn instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        None
    }

    fn each_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.cells[&cell.index()].each_inst())
    }

    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells[&cell.index()].each_dependent_cell())
    }

    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells[&cell.index()].each_cell_dependency())
    }

    fn parent(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        cell_instance.parent_cell.upgrade().unwrap()
    }

    fn template(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        cell_instance.cell().upgrade().unwrap()
    }
}


impl LayoutBase for Layout {
    type Coord = SInt;
    type LayerId = LayerIndex;
    type ShapeId = Rc<Shape<Coord>>;

    fn dbu(&self) -> Self::Coord {
        self.dbu as SInt
    }

    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_> {
        Box::new(self.layer_info.keys().copied())
    }

    fn layer_info(&self, layer: &LayerId) -> &LayerInfo {
        self.get_layer_info(*layer).expect("Non existent layer ID.")
    }

    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.find_layer(index, datatype)
    }

    fn each_shape_id<'a>(&'a self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + 'a> {
        let cell_id = cell.index();
        let layer = *layer;

        let generator = Gen::new(|co| async move {
            if let Some(shapes) = self.cells[&cell_id].shapes(layer) {
                for shape in shapes.each_shape() {
                    co.yield_(shape).await;
                }
            }
        });
        Box::new(generator.into_iter())
    }

    // fn with_shapes<'a, F, R>(& self, cell: &Self::CellId, layer: &Self::LayerId, f: F) -> R
    //     where F: FnMut(dyn IntoIterator<Item=&'a Geometry<Self::Coord>>) -> R{
    //     f(
    //         &cell.shapes(*layer).into_iter()
    //             .flat_map(|shapes| shapes.each_shape())
    //     )
    // }


    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, mut f: F)
        where F: FnMut(&Geometry<Self::Coord>) -> () {
        if let Some(shapes) = cell.shapes(*layer) {
            shapes.for_each_shape(|s| f(&s.geometry))
        }
    }
}

// pub struct GeometryIter<C: CoordinateType> {
//     shapes: Rc<Shapes<C>>
// }
//
// impl<'a, C: CoordinateType> IntoIterator for &'a GeometryIter<C> {
//     type Item = &'a Geometry<C>;
//     type IntoIter = Box<impl Iterator<Item=&'a Geometry<C>>>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         let i = self.shapes.each_shape().map(|shape| &shape.geometry);
//         i
//     }
// }

impl LayoutEdit for Layout {
    fn find_or_create_layer(&mut self, index: u32, datatype: u32) -> Self::LayerId {
        self.find_or_create_layer(index, datatype)
    }

    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId {
        self.create_and_get_cell(Some(name))
    }

    fn remove_cell(&mut self, cell_id: &Self::CellId) {
        Layout::remove_cell(self, cell_id)
    }

    fn create_cell_instance(&mut self, parent_cell: &Self::CellId, template_cell: &Self::CellId, name: Option<Self::NameType>, transform: SimpleTransform<Self::Coord>) -> Self::CellInstId {
        parent_cell.create_instance(template_cell, transform)
    }

    fn remove_cell_instance(&mut self, id: &Self::CellInstId) {
        id.parent_cell().upgrade()
            .map(|p| p.remove_cell_instance(id));
    }

    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId {
        parent_cell.shapes(*layer)
            .expect("Layer not found.")
            .insert(geometry)
    }

    fn replace_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                     shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>)
                     -> Option<Geometry<Self::Coord>> {
        unimplemented!()
    }

    fn remove_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                    shape_id: &Self::ShapeId)
                    -> Option<Geometry<Self::Coord>> {
        unimplemented!()
    }
}