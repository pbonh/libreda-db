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

use std::borrow::Borrow;
use std::hash::Hash;

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

    // /// Iterate over all cells.
    // fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Get the name of the cell.
    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType;

    /// Get the name of the cell instance.
    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType>;

    // /// Iterate over all child instance in a cell.
    // fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_>;

    // /// Iterate over all cells that contain a child of type `cell`.
    // fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_>;
    //
    // /// Iterate over all cells types that are instantiated in this `cell`.
    // fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

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

    /// Iterate over all cells that are childs of this `cell`.
    fn each_cell_dependency<'a>(&'a self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a> {
        Box::new(self.each_cell_dependency_vec(cell).into_iter())
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

    // /// Get the number of cell instances inside the `cell`.
    // fn num_child_instances(&self, cell: &Self::CellId) -> usize;
    //
    // /// Get the number of cells inside in this netlist.
    // fn num_cells(&self) -> usize;

    // /// Get the number of references that point to this cell, i.e. the number of
    // /// instances of this cell.
    // fn num_references(&self, cell: &Self::CellId) -> usize {
    //     self.each_reference(cell).count()
    // }
}