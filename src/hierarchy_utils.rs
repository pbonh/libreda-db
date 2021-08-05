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

//! Utility functions for dealing with the hierarchy of netlists or layouts.

use crate::traits::{HierarchyBase, HierarchyEdit};

/// Non-modifying utility functions for the cell hierarchy..
/// Import the this trait to use the utility functions all types that implement the `HierarchyBase` trait.
pub trait HierarchyUtil: HierarchyBase {
    /// Check if the cell is a top level cell.
    /// This is done by checking that no other cells have an instance of this cell.
    fn is_top_level_cell(&self, cell: &Self::CellId) -> bool {
        self.num_dependent_cells(cell) == 0
    }

    /// Check if the cell is a leaf cell.
    /// This is done by checking that this cell contains no other cell instances.
    fn is_leaf_cell(&self, cell: &Self::CellId) -> bool {
        self.num_cell_dependencies(cell) == 0
    }

    /// Iterate over all top level cells.
    fn each_top_level_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.each_cell()
            .filter(move |c| self.is_top_level_cell(c)))
    }
}

impl<N: HierarchyBase> HierarchyUtil for N {}

/// Modifying utility functions for the cell hierarchy..
/// Import the this trait to use the utility functions all types that implement the `HierarchyEdit` trait.
pub trait HierarchyEditUtil: HierarchyEdit {
    /// Create a named cell instance.
    /// This is just a convenience function for `create_cell_instance()`.
    fn create_named_cell_instance<S: Into<Self::NameType>>(&mut self, parent: &Self::CellId, template: &Self::CellId, name: S) -> Self::CellInstId {
        self.create_cell_instance(parent, template, Some(name.into()))
    }
    /// Create an anonymous cell instance.
    /// This is just a convenience function for `create_cell_instance()`.
    fn create_unnamed_cell_instance(&mut self, parent: &Self::CellId, template: &Self::CellId) -> Self::CellInstId {
        self.create_cell_instance(parent, template, None)
    }
}

impl<N: HierarchyEdit> HierarchyEditUtil for N {}