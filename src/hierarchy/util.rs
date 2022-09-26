// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Utility functions for dealing with the hierarchy of netlists or layouts.

use fnv::FnvHashSet;
use super::traits::{HierarchyBase, HierarchyEdit};

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

    /// Iterate over topologically sorted cells (from leaf-cells to top-cells).
    fn each_cell_bottom_to_top(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        let mut unsorted_cells: Vec<_> = self.each_cell_vec();
        let mut visited_cells: FnvHashSet<_> = Default::default();
        let mut sorted_cells = vec![];

        unsorted_cells.retain(|cell| {
            let all_dependencies_resolved = self.each_cell_dependency(cell)
                .all(|dependency| visited_cells.contains(&dependency));
            if all_dependencies_resolved {
                sorted_cells.push(cell.clone());
                visited_cells.insert(cell.clone());
                false
            } else {
                true
            }
        });

        debug_assert!({
            let mut is_topo_sorted = true;
            for (i, cell) in sorted_cells.iter().enumerate() {
                is_topo_sorted &= self.each_cell_dependency(&cell)
                    .all(|dependency| sorted_cells[0..i].contains(&dependency))
            }
            is_topo_sorted
        });

        Box::new(sorted_cells.into_iter())
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