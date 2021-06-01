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
    type NameType: Eq + Hash + From<String> + Clone + Borrow<String> + Borrow<str>;
    /// Cell/module identifier type.
    type CellId: Eq + Hash + Clone;
    /// Cell instance identifier type.
    type CellInstId: Eq + Hash + Clone;

    /// Create a new empty data structure.
    fn new() -> Self;

    /// Find a cell by its name.
    /// Return the cell with the given name. Returns `None` if the cell does not exist.
    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N>;

    /// Iterate over all cells.
    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Get the name of the cell.
    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType;

    /// Get the name of the cell instance.
    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType>;

    /// Iterate over all child instance in a cell.
    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_>;

    /// Iterate over all cells that contain a child of type `cell`.
    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Iterate over all cells types that are instantiated in this `cell`.
    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_>;

    /// Get the ID of the parent cell of this instance.
    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId;

    /// Get the ID of the template cell of this instance.
    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId;
}