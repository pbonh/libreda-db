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

//! Basic traits that for the representation of chip data structures.

#![allow(unused_variables)]

use std::borrow::Borrow;
use std::hash::Hash;
use crate::prelude::PropertyValue;

pub use crate::layout::traits::{LayoutBase, LayoutEdit};
pub use crate::netlist::traits::{NetlistBase, NetlistEdit};

/// Most basic trait for the hierarchical flyweight pattern which is
/// used to efficiently represent chip layouts and netlists.
///
/// ## Component relations
///
/// A netlist consists of cells which are templates for cell instances.
/// Each cell may contain such instances of other cell.
///
/// The following diagram illustrates how this composition graph can be traversed using the functions
/// defined by `HierarchyBase`.
///
/// ```txt
///                          each_cell_dependency
///                      +---------------------------+
///                      |                           |
///                      +                           v
///       +----------------+   each_dependent_cell  +------------------+
///       |Circuit (Top)   |<----------------------+|Circuit (Sub)     |
///       +----------------+                        +------------------+
///       |+              ^|                        | ^   +            |
///       ||each_instance ||                        | |   |            |
///       ||              ||                        | |   |            |
///       ||              |parent                   | |   |            |
///       ||              ||                        | |   |            |
///       ||+-----------+ ||                        | |   |            |
///  +--> |>|Inst1 (Sub)|-+|                        | |   |            |
///  |    ||+-----------+  |                        | |   |            |
///  |    ||               |                        | |   |            |
///  |    ||               |                        +-|---|------------+
///  |    ||               |                          |   |
///  |    ||+-----------+  |  template                |   |
///  +--> |>|Inst2 (Sub)|+----------------------------+   |
///  |    | +-----------+  |                              |
///  |    |                |                              |
///  |    |                |                              |
///  |    +----------------+                              |
///  |                                                    |
///  |                         each_reference             |
///  +----------------------------------------------------+
/// ```
///
pub trait HierarchyBase {
    /// Type for names of cells, instances, etc.
    type NameType: Eq + Hash + From<String> + Into<String> + Clone
    + Borrow<String> + Borrow<str>
    + PartialOrd + Ord
    + std::fmt::Display + std::fmt::Debug;
    /// Cell/module identifier type.
    type CellId: Eq + Hash + Clone + std::fmt::Debug;
    /// Cell instance identifier type.
    type CellInstId: Eq + Hash + Clone + std::fmt::Debug;

    /// Create a new empty data structure.
    fn new() -> Self;

    /// Find a cell by its name.
    /// Return the cell with the given name. Returns `None` if the cell does not exist.
    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N>;

    /// Find a cell instance by its name.
    /// Returns `None` if the name does not exist.
    fn cell_instance_by_name<N: ?Sized + Eq + Hash>(&self, parent_cell: &Self::CellId, name: &N) -> Option<Self::CellInstId>
        where Self::NameType: Borrow<N>;

    /// Get the name of the cell.
    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType;

    /// Get the name of the cell instance.
    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType>;

    /// Get the ID of the parent cell of this instance.
    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId;

    /// Get the ID of the template cell of this instance.
    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId;

    /// Call a function on each cell of the netlist.
    fn for_each_cell<F>(&self, f: F) where F: FnMut(Self::CellId) -> ();

    /// Get a `Vec` of all cell IDs in this netlist.
    fn each_cell_vec(&self) -> Vec<Self::CellId> {
        let mut v = Vec::new();
        self.for_each_cell(|c| v.push(c.clone()));
        v
    }

    /// Iterate over all cells.
    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.each_cell_vec().into_iter())
    }

    /// Call a function on each instance in this cell.
    fn for_each_cell_instance<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> ();

    /// Get a `Vec` of the IDs of all instances in this cell.
    fn each_cell_instance_vec(&self, cell: &Self::CellId) -> Vec<Self::CellInstId> {
        let mut v = Vec::new();
        self.for_each_cell_instance(cell, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all instances in a cell.
    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.each_cell_instance_vec(cell).into_iter())
    }

    /// Call a function for each cell that is a child of this `cell`.
    fn for_each_cell_dependency<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> ();

    /// Get a `Vec` of each cell that is a child of this `cell`.
    fn each_cell_dependency_vec(&self, cell: &Self::CellId) -> Vec<Self::CellId> {
        let mut v = Vec::new();
        self.for_each_cell_dependency(cell, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all cells that are instantiated in this `cell`.
    fn each_cell_dependency<'a>(&'a self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a> {
        Box::new(self.each_cell_dependency_vec(cell).into_iter())
    }

    /// Count all cells that are dependencies of `cell`.
    fn num_cell_dependencies(&self, cell: &Self::CellId) -> usize {
        // Inefficient default implementation.
        let mut counter = 0;
        self.for_each_cell_dependency(cell, |_| counter += 1);
        counter
    }

    /// Call a function for each cell that directly depends on `cell`.
    fn for_each_dependent_cell<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> ();

    /// Get a `Vec` of each cell that directly depends on `cell`.
    fn each_dependent_cell_vec(&self, cell: &Self::CellId) -> Vec<Self::CellId> {
        let mut v = Vec::new();
        self.for_each_dependent_cell(cell, |c| v.push(c.clone()));
        v
    }

    /// Iterate over each cell that directly depends on `cell`.
    fn each_dependent_cell<'a>(&'a self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a> {
        Box::new(self.each_dependent_cell_vec(cell).into_iter())
    }

    /// Count all cells that are directly dependent on `cell`, i.e. contain an instance of `cell`.
    fn num_dependent_cells(&self, cell: &Self::CellId) -> usize {
        // Inefficient default implementation.
        let mut counter = 0;
        self.for_each_cell_dependency(cell, |_| counter += 1);
        counter
    }

    /// Iterate over all instances of this `cell`, i.e. instances that use this cell as
    /// a template.
    fn for_each_cell_reference<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> ();

    /// Get a `Vec` with all cell instances referencing this cell.
    fn each_cell_reference_vec(&self, cell: &Self::CellId) -> Vec<Self::CellInstId> {
        let mut v = Vec::new();
        self.for_each_cell_reference(cell, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all instances of this `cell`, i.e. instances that use this cell as
    /// a template.
    fn each_cell_reference(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        // Provide an inefficient default implementation.
        Box::new(self.each_cell_reference_vec(cell).into_iter())
    }

    /// Count all reference to the cell template `cell`.
    fn num_cell_references(&self, cell: &Self::CellId) -> usize {
        // Inefficient default implementation.
        let mut counter = 0;
        self.for_each_cell_reference(cell, |_| counter += 1);
        counter
    }

    /// Get the number of cell instances inside the `cell`.
    fn num_child_instances(&self, cell: &Self::CellId) -> usize;

    /// Get the number of cell templates.
    fn num_cells(&self) -> usize;

    /// Get a property of the top-level chip data structure..
    fn get_chip_property(&self, key: &Self::NameType) -> Option<PropertyValue> {
        None
    }

    /// Get a property of a cell.
    fn get_cell_property(&self, cell: &Self::CellId, key: &Self::NameType) -> Option<PropertyValue> {
        None
    }

    /// Get a property of a cell instance.
    fn get_cell_instance_property(&self, inst: &Self::CellInstId, key: &Self::NameType) -> Option<PropertyValue> {
        None
    }
}

/// Edit functions for a hierarchical flyweight structure like a netlist or a cell-based layout.
pub trait HierarchyEdit: HierarchyBase {

    /// Create a new and empty cell template.
    /// A cell template can be be instantiated in other cells.
    ///
    /// # Example
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut chip = Chip::new();
    /// let my_cell = chip.create_cell("myCell".into());
    ///
    /// assert_eq!(chip.num_cells(), 1);
    /// assert_eq!(chip.cell_by_name("myCell"), Some(my_cell));
    /// ```
    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId;

    /// Remove a cell and all the instances of it.
    ///
    /// # Example
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut chip = Chip::new();
    /// let top = chip.create_cell("TOP".into());
    /// assert_eq!(chip.num_cells(), 1);
    /// chip.remove_cell(&top);
    /// assert_eq!(chip.num_cells(), 0);
    /// ```
    fn remove_cell(&mut self, cell_id: &Self::CellId);

    /// Create a new instance of `template_cell` in `parent_cell`.
    /// Recursive instantiation is forbidden and might panic.
    ///
    /// # Example
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut chip = Chip::new();
    /// let top = chip.create_cell("TOP".into());
    /// let sub = chip.create_cell("SUB".into());
    ///
    /// // Create two instances of "SUB" inside "TOP".
    /// let inst1 = chip.create_cell_instance(&top, &sub, Some("sub1".into())); // Create named instance.
    /// let inst2 = chip.create_cell_instance(&top, &sub, None); // Create unnamed instance.
    ///
    /// assert_eq!(chip.num_child_instances(&top), 2);
    /// assert_eq!(chip.num_cell_references(&sub), 2);
    /// ```
    fn create_cell_instance(&mut self,
                            parent_cell: &Self::CellId,
                            template_cell: &Self::CellId,
                            name: Option<Self::NameType>) -> Self::CellInstId;

    /// Remove cell instance if it exists.
    /// # Example
    /// ```
    /// use libreda_db::prelude::*;
    /// let mut chip = Chip::new();
    /// let top = chip.create_cell("TOP".into());
    /// let sub = chip.create_cell("SUB".into());
    ///
    /// // Create two instances of "SUB" inside "TOP".
    /// let inst1 = chip.create_cell_instance(&top, &sub, Some("sub1".into())); // Create named instance.
    /// let inst2 = chip.create_cell_instance(&top, &sub, None); // Create unnamed instance.
    ///
    /// assert_eq!(chip.num_child_instances(&top), 2);
    /// assert_eq!(chip.num_cell_references(&sub), 2);
    ///
    /// chip.remove_cell_instance(&inst2);
    ///
    /// assert_eq!(chip.num_child_instances(&top), 1);
    /// assert_eq!(chip.num_cell_references(&sub), 1);
    /// ```
    fn remove_cell_instance(&mut self, inst: &Self::CellInstId);

    /// Change the name of a cell instance.
    ///
    /// Clears the name when `None` is passed.
    ///
    /// # Panics
    /// Panics if an instance with this name already exists in the parent cell.
    fn rename_cell_instance(&mut self, inst: &Self::CellInstId, new_name: Option<Self::NameType>);


    /// Change the name of a cell.
    ///
    /// # Panics
    /// Panics if a cell with this name already exists.
    fn rename_cell(&mut self, cell: &Self::CellId, new_name: Self::NameType);

    /// Set a property of the top-level chip data structure..
    fn set_chip_property(&mut self, key: Self::NameType, value: PropertyValue) {}

    /// Set a property of a cell.
    fn set_cell_property(&mut self, cell: &Self::CellId, key: Self::NameType, value: PropertyValue) {}

    /// Set a property of a cell instance.
    fn set_cell_instance_property(&mut self, inst: &Self::CellInstId, key: Self::NameType, value: PropertyValue) {}
}

/// Fused layout and netlist view.
/// This trait makes the link between netlist elements and layout elements.
pub trait L2NBase: LayoutBase + NetlistBase {
    /// Iterate over all shapes that are marked to belong to the specified net.
    fn shapes_of_net(&self, net_id: &Self::NetId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_>;
    /// Iterate over all shapes that are part of the pin.
    fn shapes_of_pin(&self, pin_id: &Self::PinId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_>;
    /// Get the net of a shape.
    fn get_net_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::NetId>;
    /// Get the pin that belongs to the shape (if any).
    fn get_pin_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::PinId>;
}

/// Fused layout and netlist view.
/// This trait makes the link between netlist elements and layout elements.
pub trait L2NEdit: L2NBase + LayoutEdit + NetlistEdit{
    /// Create the link between a circuit pin and its shapes in the layout.
    /// Return the current pin.
    fn set_pin_of_shape(&mut self, shape_id: &Self::ShapeId, pin: Option<Self::PinId>) -> Option<Self::PinId>;
    /// Set the net of a shape.
    /// Return the current net.
    fn set_net_of_shape(&mut self, shape_id: &Self::ShapeId, net: Option<Self::NetId>) -> Option<Self::NetId>;
}