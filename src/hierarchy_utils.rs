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

    /// Iterate over all leaf cells, i.e. cells which contain no other cells.
    fn each_leaf_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.each_cell()
            .filter(move |c| self.is_leaf_cell(c)))
    }
}

impl<N: HierarchyBase> HierarchyUtil for N {}

/// Modifying utility functions for the cell hierarchy..
/// Import the this trait to use the utility functions all types that implement the `HierarchyEdit` trait.
pub trait HierarchyEditUtil: HierarchyEdit {

    /// Remove all unused cells, i.e. cells that are not instantiated.
    /// Return the number of removed cells.
    fn prune_cells(&mut self) -> usize {
        // Get a list of all unused cells.
        let unused_cells: Vec<_> = self.each_cell()
            .filter(|cell_id| self.num_cell_references(cell_id) == 0)
            .collect();

        // Remove them.
        for unused in &unused_cells {
            self.remove_cell(&unused)
        }

        unused_cells.len()
    }
}

impl<N: HierarchyEdit> HierarchyEditUtil for N {}