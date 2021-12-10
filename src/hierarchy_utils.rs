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
    /// Remove all child instances inside the `cell`.
    fn clear_cell_instances(&mut self, cell: &Self::CellId) {
        let child_instances = self.each_cell_instance_vec(cell);
        for child in &child_instances {
            self.remove_cell_instance(child);
        }
    }

    /// Remove the cell instance and all cells which are then not used anymore.
    fn prune_cell_instance(&mut self, inst: &Self::CellInstId) {
        let template = self.template_cell(inst);
        self.remove_cell_instance(inst);
        if self.num_cell_references(&template) == 0 {
            // The cell is now not used anymore.
            self.remove_cell(&template)
        }
    }

    /// Remove the cell and all other cells which are then not used anymore.
    fn prune_cell(&mut self, cell: &Self::CellId) {
        let child_instances = self.each_cell_instance_vec(cell);
        for child in &child_instances {
            self.prune_cell_instance(child);
        }
        self.remove_cell(&cell)
    }
}

impl<N: HierarchyEdit> HierarchyEditUtil for N {}