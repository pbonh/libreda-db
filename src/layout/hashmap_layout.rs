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
use crate::index::*;
use crate::layout::types::*;
use crate::property_storage::PropertyStore;
use crate::rc_string::RcString;
use iron_shapes::CoordinateType;
use iron_shapes::shape::Geometry;
use iron_shapes::transform::SimpleTransform;
use crate::prelude::{LayoutBase, LayoutEdit, HierarchyEdit, HierarchyBase};
use std::hash::Hash;
use std::borrow::Borrow;

// Use an alternative hasher that has good performance for integer keys.
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::HashMap;
use std::ops::Deref;

type IntHashMap<K, V> = FnvHashMap<K, V>;
type IntHashSet<V> = FnvHashSet<V>;

/// Cell identifier.
pub type CellId<T> = Index<Cell<T>>;

/// Cell instance identifier.
pub type CellInstId<T> = Index<CellInstance<T>>;

/// Unique (across layout) identifier of a shape.
pub type ShapeId<T> = Index<Shape<T>>;

/// ID for layers.
pub type LayerId = Index<LayerInfo>;

/// Data structure which holds cells and cell instances.
///
/// # Examples
///
/// ```
/// use libreda_db::prelude::*;
/// let layout = Layout::new();
/// ```
#[derive(Default, Debug)]
pub struct Layout<C: CoordinateType> {
    /// Data-base unit. Pixels per micrometer.
    dbu: C,
    /// All cell templates.
    cells: IntHashMap<CellId<C>, Cell<C>>,
    /// All cell instances.
    cell_instances: IntHashMap<CellInstId<C>, CellInstance<C>>,
    /// Counter for generating the next cell index.
    cell_index_generator: IndexGenerator<Cell<C>>,
    /// Counter for generating the next cell instance index.
    cell_instance_index_generator: IndexGenerator<CellInstance<C>>,

    /// ID generator for shapes.
    shape_index_generator: IndexGenerator<Shape<C>>,

    /// Lookup table for finding cells by name.
    cells_by_name: HashMap<RcString, CellId<C>>,
    /// Counter for generating the next layer index.
    layer_index_generator: IndexGenerator<LayerInfo>,
    /// Lookup table for finding layers by name.
    layers_by_name: HashMap<RcString, LayerId>,
    /// Lookup table for finding layers by index/datatype numbers.
    layers_by_index_datatype: IntHashMap<(UInt, UInt), LayerId>,
    /// Info structures for all layers.
    layer_info: IntHashMap<LayerId, LayerInfo>,
    /// Property storage for properties related to this layout.
    property_storage: PropertyStore<RcString>,
}

impl<C: CoordinateType> Layout<C> {
    /// Get a reference to the cell with the given `ID`.
    ///
    /// # Panics
    /// Panics if the ID does not exist.
    pub fn cell_by_id(&self, id: &CellId<C>) -> CellRef<'_, C> {
        let cell = &self.cells[id];
        CellRef {
            layout: self,
            cell,
        }
    }

    /// Find the ID of the cell with the given `name`.
    pub fn cell_id_by_name(&self, name: &str) -> Option<CellId<C>> {
        self.cells_by_name.get(name).copied()
    }

    /// Get a reference to the cell with the given name.
    pub fn cell_by_name(&self, name: &str) -> Option<CellRef<'_, C>> {
        let cell_id = self.cell_id_by_name(name);

        cell_id.map(|cell_id| {
            let cell = &self.cells[&cell_id];
            CellRef {
                layout: self,
                cell,
            }
        })
    }

    /// Iterate over all cells in this layout.
    pub fn each_cell(&self) -> impl Iterator<Item=CellRef<'_, C>> {
        self.cells.values()
            .map(move |cell| CellRef {
                layout: self,
                cell,
            })
    }
}

/// A `Cell` is a container for geometrical shapes organized on different layers.
/// Additionally to the geometrical shapes a cell can also contain instances of other cells.
#[derive(Clone, Debug)]
pub struct Cell<C: CoordinateType, U = ()> {
    /// Cell name.
    name: RcString,
    /// The index of this cell inside the layout. This is none if the cell does not belong to a layout.
    index: CellId<C>,
    /// Child cells.
    cell_instances: IntHashSet<CellInstId<C>>,

    /// Cell instances indexed by name.
    cell_instances_by_name: HashMap<RcString, CellInstId<C>>,

    /// Mapping from layer indices to geometry data.
    shapes_map: IntHashMap<LayerId, Shapes<C>>,

    /// All the instances of this cell.
    cell_references: IntHashSet<CellInstId<C>>,

    /// Set of cells that are dependencies of this cell.
    /// Stored together with a counter of how many instances of the dependency are present.
    /// This are the cells towards the leaves in the dependency tree.
    dependencies: IntHashMap<CellId<C>, usize>,
    /// Cells that use an instance of this cell.
    /// This are the cells towards the root in the dependency tree.
    dependent_cells: IntHashMap<CellId<C>, usize>,
    /// Properties related to this cell.
    cell_properties: PropertyStore<RcString>,
    /// Properties related to the instances in this cell.
    /// Instance properties are stored here for lower overhead of cell instances.
    instance_properties: IntHashMap<CellInstId<C>, PropertyStore<RcString>>,
    /// User-defined data.
    user_data: U,
}

impl<C: CoordinateType> Cell<C> {
    /// Get the name of this cell.
    pub fn name(&self) -> &RcString {
        &self.name
    }

    /// Get the ID of this cell.
    /// The ID uniquely identifies the cell within this layout.
    pub fn id(&self) -> CellId<C> {
        self.index
    }

    /// Get the shape container of this layer.
    /// Returns `None` if the shapes object does not exist for this layer.
    pub fn shapes(&self, layer_id: &LayerId) -> Option<&Shapes<C>> {
        self.shapes_map.get(layer_id)
    }

    /// Find a child instance in this cell by its name.
    pub fn instance_id_by_name(&self, name: &str) -> Option<CellInstId<C>> {
        self.cell_instances_by_name.get(name).copied()
    }

    /// Iterate over the IDs of each layer of each used layer in this cell.
    pub fn each_used_layer(&self) -> impl Iterator<Item=LayerId> + '_ {
        self.shapes_map.keys().copied()
    }

    /// Iterate over the IDs of the child cell instances.
    pub fn each_instance_id(&self) -> impl Iterator<Item=CellInstId<C>> + '_ {
        self.cell_instances.iter().copied()
    }

    /// Iterate over the IDs of each dependency of this cell.
    /// A dependency is a cell that is instantiated in `self`.
    pub fn each_dependency_id(&self) -> impl Iterator<Item=CellId<C>> + '_ {
        self.dependencies.keys().copied()
    }

    /// Iterate over the IDs of cell that depends on this cell.
    pub fn each_dependent_cell_id(&self) -> impl Iterator<Item=CellId<C>> + '_ {
        self.dependent_cells.keys().copied()
    }
}

/// A 'fat' reference to a cell.
///
/// This struct also keeps a reference to a cell and to the layout.
///
/// This allows convenient read-only access to the layout in an object like manner.
#[derive(Copy, Clone, Debug)]
pub struct CellRef<'a, C: CoordinateType> {
    /// Reference to the parent layout.
    layout: &'a Layout<C>,
    /// Reference to the cell.
    cell: &'a Cell<C>,
}

/// All functions of `Cell` are made available also for `CellRef` by implementation of the `Deref` trait.
impl<'a, C: CoordinateType> Deref for CellRef<'a, C> {
    type Target = Cell<C>;

    fn deref(&self) -> &Self::Target {
        self.cell
    }
}

impl<C: CoordinateType> CellRef<'_, C> {
    /// Iterate over all cell instances in this cell.
    pub fn each_instance_ref(&self) -> impl Iterator<Item=CellInstanceRef<'_, C>> {
        self.cell_instances.iter()
            .map(move |inst_id| {
                let inst = &self.layout.cell_instances[inst_id];
                CellInstanceRef {
                    layout: self.layout,
                    inst,
                }
            })
    }

    /// Find a child cell instance by its name.
    /// Returns `None` if no such instance exists.
    pub fn instance_ref_by_name(&self, name: &str) -> Option<CellInstanceRef<'_, C>> {
        let id = self.instance_id_by_name(name);
        id.map(|id| {
            let inst = &self.layout.cell_instances[&id];
            CellInstanceRef {
                layout: self.layout,
                inst,
            }
        })
    }

    /// Iterate over the references to all cells that are dependencies of this cell.
    pub fn each_dependency_ref(&self) -> impl Iterator<Item=CellRef<'_, C>> {
        self.each_dependency_id()
            .map(move |id| CellRef {
                layout: self.layout,
                cell: &self.layout.cells[&id],
            })
    }

    /// Iterate over the references to all cells that are dependent on this cell.
    pub fn each_dependent_cell_ref(&self) -> impl Iterator<Item=CellRef<'_, C>> {
        self.each_dependent_cell_id()
            .map(move |id| CellRef {
                layout: self.layout,
                cell: &self.layout.cells[&id],
            })
    }
}

/// An actual instance of a cell.
#[derive(Clone, Debug)]
pub struct CellInstance<C: CoordinateType, U = ()> {
    /// Name of the instance.
    name: Option<RcString>,
    /// ID of the parent cell.
    parent_cell_id: CellId<C>,
    /// Identifier. Uniquely identifies the instance within the parent cell.
    id: CellInstId<C>,
    /// ID of the template cell.
    template_cell_id: CellId<C>,
    /// Transformation to put the cell to the right place an into the right scale/rotation.
    transform: SimpleTransform<C>,
    // TODO: Repetition
    /// User-defined data.
    user_data: U,
}


impl<C: CoordinateType> CellInstance<C> {
    /// Get the name of this cell instance.
    pub fn name(&self) -> &Option<RcString> {
        &self.name
    }

    /// Get the ID of this cell instance.
    /// The ID uniquely identifies the cell within this layout.
    pub fn id(&self) -> CellInstId<C> {
        self.id
    }

    /// Get the ID of the parent cell.
    pub fn parent_cell_id(&self) -> CellId<C> {
        self.parent_cell_id
    }

    /// Get the ID of the template cell.
    pub fn template_cell_id(&self) -> CellId<C> {
        self.template_cell_id
    }

    /// Get the transformation that represents the location and orientation of this instance.
    pub fn get_transform(&self) -> &SimpleTransform<C> {
        &self.transform
    }
}

/// A reference to a cell instance.
///
/// This struct also keeps a reference to the parent layout struct of the cell.
#[derive(Copy, Clone, Debug)]
pub struct CellInstanceRef<'a, C: CoordinateType> {
    layout: &'a Layout<C>,
    inst: &'a CellInstance<C>,
}

impl<'a, C: CoordinateType> Deref for CellInstanceRef<'a, C> {
    type Target = CellInstance<C>;

    fn deref(&self) -> &Self::Target {
        self.inst
    }
}

impl<C: CoordinateType> CellInstanceRef<'_, C> {
    /// Get reference to the layout struct where this cell instance lives in.
    pub fn layout(&self) -> &Layout<C> {
        self.layout
    }

    /// Get a reference to the parent cell of this instance.
    pub fn parent_cell(&self) -> CellRef<C> {
        let parent = &self.layout.cells[&self.parent_cell_id];
        CellRef {
            layout: self.layout,
            cell: parent,
        }
    }

    /// Get a reference to the template cell of this instance.
    pub fn template_cell(&self) -> CellRef<C> {
        let template = &self.layout.cells[&self.template_cell_id];
        CellRef {
            layout: self.layout,
            cell: template,
        }
    }
}

/// Wrapper around a `Geometry` struct.
#[derive(Clone, Debug)]
pub struct Shape<T: CoordinateType, U = ()> {
    /// Identifier of this shape.
    index: Index<Self>,
    /// The geometry of this shape.
    pub geometry: Geometry<T>,
    // /// Reference ID to container.
    // parent_id: Index<Shapes<T>>,
    /// User-defined data.
    user_data: U,
}

/// `Shapes<T>` is a collection of `Shape<T>` structs. Each of
/// the elements is assigned an index when inserted into the collection.
#[derive(Clone, Debug)]
pub struct Shapes<C>
    where C: CoordinateType {
    /// ID of this shape collection.
    id: Index<Self>,
    /// Reference to the cell where this shape collection lives. Can be none.
    parent_cell: CellId<C>,
    /// Shape elements.
    shapes: IntHashMap<ShapeId<C>, Shape<C>>,
    /// Property stores for the shapes.
    shape_properties: IntHashMap<ShapeId<C>, PropertyStore<RcString>>,
}

impl<C: CoordinateType> Shapes<C> {
    /// Get the ID of this shape container.
    pub fn id(&self) -> Index<Self> {
        self.id
    }

    /// Iterate over all geometric shapes in this collection.
    pub fn each_shape(&self) -> impl Iterator<Item=&Shape<C>> {
        self.shapes.values()
    }
}

impl<C: CoordinateType> HierarchyBase for Layout<C> {
    type NameType = RcString;
    type CellId = CellId<C>;
    type CellInstId = CellInstId<C>;

    /// Create a new empty layout.
    fn new() -> Self {
        Layout {
            dbu: C::zero(),
            cells: Default::default(),
            cell_instances: Default::default(),
            cell_index_generator: Default::default(),
            cell_instance_index_generator: Default::default(),
            shape_index_generator: Default::default(),
            cells_by_name: Default::default(),
            layer_index_generator: Default::default(),
            layers_by_name: Default::default(),
            layers_by_index_datatype: Default::default(),
            layer_info: Default::default(),
            property_storage: Default::default(),
        }
    }

    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N> {
        self.cells_by_name.get(name).copied()
    }

    fn cell_instance_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::CellInstId>
        where Self::NameType: Borrow<N> {
        self.cells[parent_circuit].cell_instances_by_name.get(name).copied()
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        self.cells[cell].name.clone()
    }

    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.cell_instances[cell_inst].name.clone()
    }

    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.cell_instances[cell_instance].parent_cell_id
    }

    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.cell_instances[cell_instance].template_cell_id
    }

    fn for_each_cell<F>(&self, mut f: F) where F: FnMut(Self::CellId) -> () {
        self.cells.keys().for_each(|&id| f(id))
    }

    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells.keys().copied())
    }

    fn for_each_cell_instance<F>(&self, cell: &Self::CellId, mut f: F) where F: FnMut(Self::CellInstId) -> () {
        self.cells[cell].cell_instances.iter().for_each(|&id| f(id))
    }

    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.cells[cell].cell_instances.iter().copied())
    }

    fn for_each_cell_dependency<F>(&self, circuit: &Self::CellId, mut f: F) where F: FnMut(Self::CellId) -> () {
        self.cells[circuit].dependencies.keys().for_each(|&id| f(id))
    }

    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells[cell].dependencies.keys().copied())
    }

    fn for_each_dependent_cell<F>(&self, circuit: &Self::CellId, mut f: F) where F: FnMut(Self::CellId) -> () {
        self.cells[circuit].dependent_cells.keys().for_each(|&id| f(id))
    }

    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells[cell].dependent_cells.keys().copied())
    }

    fn for_each_cell_reference<F>(&self, circuit: &Self::CellId, mut f: F) where F: FnMut(Self::CellInstId) -> () {
        self.cells[circuit].cell_references.iter().for_each(|&id| f(id))
    }
}


impl<C: CoordinateType> LayoutBase for Layout<C> {
    type Coord = C;
    type LayerId = LayerId;
    type ShapeId = ShapeId<C>;


    fn dbu(&self) -> Self::Coord {
        self.dbu
    }

    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_> {
        Box::new(self.layer_info.keys().copied())
    }

    fn layer_info(&self, layer: &LayerId) -> &LayerInfo {
        &self.layer_info[layer]
    }

    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.layers_by_index_datatype.get(&(index, datatype)).copied()
    }

    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        Box::new(self.cells[cell].shapes_map[layer].shapes.values().map(|s| s.index))
    }

    // fn each_shape(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=&Geometry<Self::Coord>> + '_> {
    //     Box::new(self.cells[cell].shapes_map[layer].shapes.values().map(|s| &s.geometry))
    // }

    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, mut f: F)
        where F: FnMut(&Geometry<Self::Coord>) -> () {
        self.cells[cell].shapes_map[layer].shapes.values()
            .for_each(|s| f(&s.geometry))
    }

    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<Self::Coord> {
        self.cell_instances[cell_inst].transform.clone()
    }
}

impl<C: CoordinateType> HierarchyEdit for Layout<C> {
    fn create_cell(&mut self, name: RcString) -> CellId<C> {
        assert!(!self.cells_by_name.contains_key(&name), "Cell with this name already exists.");
        let id = self.cell_index_generator.next();

        let cell = Cell {
            name: name.clone(),
            index: id,
            cell_instances: Default::default(),
            cell_instances_by_name: Default::default(),
            shapes_map: Default::default(),
            cell_references: Default::default(),
            dependencies: Default::default(),
            dependent_cells: Default::default(),
            cell_properties: Default::default(),
            instance_properties: Default::default(),
            user_data: Default::default(),
        };

        self.cells.insert(id, cell);
        self.cells_by_name.insert(name, id);

        id
    }

    fn remove_cell(&mut self, cell_id: &CellId<C>) {
        // Remove all instances inside this cell.
        let instances = self.cells[cell_id].cell_instances.iter().copied().collect_vec();
        for inst in instances {
            self.remove_cell_instance(&inst);
        }
        // Remove all instances of this cell.
        let references = self.cells[cell_id].cell_references.iter().copied().collect_vec();
        for inst in references {
            self.remove_cell_instance(&inst);
        }

        // Remove the cell.
        let name = self.cells[cell_id].name.clone();
        self.cells_by_name.remove(&name).unwrap();
        self.cells.remove(&cell_id).unwrap();
    }

    /// Create a new cell instance at location (0, 0).
    fn create_cell_instance(&mut self, parent_cell: &CellId<C>,
                            template_cell: &CellId<C>,
                            name: Option<RcString>) -> CellInstId<C> {

        let id = self.cell_instance_index_generator.next();

        // Default location is (0, 0), no magnification, no rotation or mirroring.
        let transform = SimpleTransform::identity();

        {
            // Check that creating this cell instance does not create a cycle in the dependency graph.
            // There can be no recursive instances.
            let mut stack: Vec<CellId<C>> = vec![*parent_cell];
            while let Some(c) = stack.pop() {
                if &c == template_cell {
                    // The cell to be instantiated depends on the current cell.
                    // This would insert a loop into the dependency tree.
                    // TODO: Don't panic but return an `Err`.
                    panic!("Cannot create recursive instances.");
                }
                // Follow the dependent cells towards the root.
                stack.extend(self.cells[&c].dependent_cells.keys().copied())
            }
        }


        let inst = CellInstance {
            name: name.clone(),
            parent_cell_id: *parent_cell,
            id: id,
            template_cell_id: *template_cell,
            transform: transform,
            user_data: Default::default(),
        };

        self.cell_instances.insert(id, inst);
        self.cells.get_mut(parent_cell).unwrap().cell_instances.insert(id);
        self.cells.get_mut(template_cell).unwrap().cell_references.insert(id);

        if let Some(name) = name {
            debug_assert!(!self.cells[parent_cell].cell_instances_by_name.contains_key(&name),
                          "Cell instance name already exists.");
            self.cells.get_mut(parent_cell).unwrap().cell_instances_by_name.insert(name, id);
        }

        // Remember dependency.
        {
            self.cells.get_mut(parent_cell).unwrap()
                .dependencies.entry(*template_cell)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        // Remember dependency.
        {
            self.cells.get_mut(template_cell).unwrap()
                .dependent_cells.entry(*parent_cell)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        id
    }

    fn remove_cell_instance(&mut self, id: &CellInstId<C>) {

        // Remove the instance and all references.
        let parent = self.cell_instances[id].parent_cell_id;
        let template = self.cell_instances[id].template_cell_id;

        // Remove dependency.
        {
            // Decrement counter.
            let parent_mut = self.cells.get_mut(&parent).unwrap();
            let count = parent_mut.dependencies.entry(template)
                .or_insert(0); // Should not happen.
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                parent_mut.dependencies.remove(&template);
            }
        }

        // Remove dependency.
        {
            // Decrement counter.
            let template_mut = self.cells.get_mut(&template).unwrap();
            let count = template_mut.dependent_cells.entry(parent)
                .or_insert(0); // Should not happen.
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                template_mut.dependent_cells.remove(&parent);
            }
        }

        self.cell_instances.remove(&id).unwrap();
        self.cells.get_mut(&parent).unwrap().cell_instances.remove(id);
        self.cells.get_mut(&template).unwrap().cell_references.remove(id);
    }

    fn rename_cell(&mut self, _cell: &Self::CellId, _new_name: Self::NameType) {
        unimplemented!()
    }
}


impl<C: CoordinateType> LayoutEdit for Layout<C> {
    fn find_or_create_layer(&mut self, index: u32, datatype: u32) -> Self::LayerId {
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

    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId {
        let shape_id = self.shape_index_generator.next();

        let shape = Shape {
            index: shape_id,
            geometry,
            user_data: Default::default(),
        };

        self.cells.get_mut(parent_cell).expect("Cell not found.")
            .shapes_map.get_mut(layer).expect("Layer not found.")
            .shapes.insert(shape_id, shape);

        shape_id
    }

    fn remove_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, shape_id: &Self::ShapeId)
                    -> Option<Geometry<Self::Coord>> {
        self.cells.get_mut(parent_cell).expect("Cell not found.")
            .shapes_map.get_mut(layer).expect("Layer not found.")
            .shapes.remove(shape_id)
            .map(|s| s.geometry)
    }

    fn replace_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                     shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>)
                     -> Option<Geometry<Self::Coord>> {
        let shape_id = *shape_id;
        let shape = Shape {
            index: shape_id,
            geometry,
            user_data: Default::default(),
        };

        self.cells.get_mut(parent_cell).expect("Cell not found.")
            .shapes_map.get_mut(layer).expect("Layer not found.")
            .shapes.insert(shape_id, shape)
            .map(|s| s.geometry)
    }

    fn set_transform(&mut self, cell_inst: &Self::CellInstId, tf: SimpleTransform<Self::Coord>) {
        self.cell_instances.get_mut(cell_inst).unwrap().transform = tf
    }
}