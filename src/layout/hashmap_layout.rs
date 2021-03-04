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
use std::collections::{HashMap, HashSet};
use crate::index::*;
use crate::layout::types::*;
use crate::property_storage::PropertyStore;
use crate::rc_string::RcString;
use iron_shapes::CoordinateType;
use iron_shapes::shape::Geometry;
use iron_shapes::transform::SimpleTransform;
use crate::layout::traits::{LayoutBase, LayoutEdit};
use std::hash::Hash;
use std::borrow::Borrow;

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
    dbu: UInt,
    /// All cell templates.
    cells: HashMap<CellId<C>, Cell<C>>,
    /// All cell instances.
    cell_instances: HashMap<CellInstId<C>, CellInstance<C>>,
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
    layers_by_index_datatype: HashMap<(UInt, UInt), LayerId>,
    /// Info structures for all layers.
    layer_info: HashMap<LayerId, LayerInfo>,
    /// Property storage for properties related to this layout.
    property_storage: PropertyStore<RcString>,
}

/// Meta-data of a layer.
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct LayerInfo {
    /// Identifier of the layer.
    pub index: UInt,
    /// Identifier of the layer.
    pub datatype: UInt,
    /// Name of the layer.
    pub name: Option<RcString>,
}

/// A `Cell` is a container for geometrical shapes organized on different layers.
/// Additionally to the geometrical shapes a cell can also contain instances of other cells.
#[derive(Clone, Debug)]
pub struct Cell<C: CoordinateType> {
    /// Cell name.
    name: RcString,
    /// The index of this cell inside the layout. This is none if the cell does not belong to a layout.
    index: CellId<C>,
    /// Child cells.
    cell_instances: HashSet<CellInstId<C>>,

    /// Cell instances indexed by name.
    cell_instances_by_name: HashMap<RcString, CellInstId<C>>,

    /// Mapping from layer indices to geometry data.
    shapes_map: HashMap<LayerId, Shapes<C>>,

    /// All the instances of this cell.
    cell_references: HashSet<CellInstId<C>>,

    /// Set of cells that are dependencies of this cell.
    /// Stored together with a counter of how many instances of the dependency are present.
    /// This are the cells towards the leaves in the dependency tree.
    dependencies: HashMap<CellId<C>, usize>,
    /// Cells that use an instance of this cell.
    /// This are the cells towards the root in the dependency tree.
    dependent_cells: HashMap<CellId<C>, usize>,
    /// Properties related to this cell.
    cell_properties: PropertyStore<RcString>,
    /// Properties related to the instances in this cell.
    /// Instance properties are stored here for lower overhead of cell instances.
    instance_properties: HashMap<CellInstId<C>, PropertyStore<RcString>>,
}

/// An actual instance of a cell.
#[derive(Clone, Debug)]
pub struct CellInstance<C: CoordinateType> {
    /// Name of the instance.
    name: Option<RcString>,
    /// ID of the parent cell.
    parent_cell_id: CellId<C>,
    /// Identifier. Uniquely identifies the instance within the parent cell.
    id: CellInstId<C>,
    /// ID of the template cell.
    template_cell: CellId<C>,
    /// Transformation to put the cell to the right place an into the right scale/rotation.
    transform: SimpleTransform<C>,
    // TODO: Repetition
}

/// Wrapper around a `Geometry` struct.
#[derive(Clone, Debug)]
pub struct Shape<T: CoordinateType> {
    /// Identifier of this shape.
    index: Index<Self>,
    /// The geometry of this shape.
    pub geometry: Geometry<T>,
    // /// Reference ID to container.
    // parent_id: Index<Shapes<T>>,
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
    shapes: HashMap<ShapeId<C>, Shape<C>>,
    /// Property stores for the shapes.
    shape_properties: HashMap<ShapeId<C>, PropertyStore<RcString>>,
}

impl<C: CoordinateType> LayoutBase for Layout<C> {
    type Coord = C;
    type NameType = RcString;
    type LayerId = LayerId;
    type CellId = CellId<C>;
    type CellInstId = CellInstId<C>;

    fn new() -> Self {
        Layout {
            dbu: 0,
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

    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells.keys().copied())
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        self.cells[cell].name.clone()
    }

    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.cell_instances[cell_inst].name.clone()
    }

    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.cells[cell].cell_instances.iter().copied())
    }

    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells[cell].dependent_cells.keys().copied())
    }

    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.cells[cell].dependencies.keys().copied())
    }

    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.cell_instances[cell_instance].parent_cell_id
    }

    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.cell_instances[cell_instance].template_cell
    }

    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.layers_by_index_datatype.get(&(index, datatype)).copied()
    }

    // fn each_shape(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=&Geometry<Self::Coord>> + '_> {
    //     Box::new(self.cells[cell].shapes_map[layer].shapes.values().map(|s| &s.geometry))
    // }

    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, mut f: F)
        where F: FnMut(&Geometry<Self::Coord>) -> () {
        self.cells[cell].shapes_map[layer].shapes.values()
            .for_each(|s| f(&s.geometry))
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

    fn create_cell_instance(&mut self, parent_cell: &CellId<C>,
                            template_cell: &CellId<C>,
                            name: Option<RcString>,
                            transform: SimpleTransform<C>) -> CellInstId<C> {
        let id = self.cell_instance_index_generator.next();

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
            template_cell: *template_cell,
            transform: transform,
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
        let template = self.cell_instances[id].template_cell;

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

    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) {
        let shape_id = self.shape_index_generator.next();

        let shape = Shape {
            index: shape_id,
            geometry,
        };

        self.cells.get_mut(parent_cell).expect("Cell not found.")
            .shapes_map.get_mut(layer).expect("Layer not found.")
            .shapes.insert(shape_id, shape);
    }
}