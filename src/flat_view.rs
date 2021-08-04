/*
 * Copyright (c) 2020-2021 Thomas Kramer.
 *
 * This file is part of LibrEDA
 * (see https://codeberg.org/libreda/arboreus-cts).
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

//! Wrapper around a netlist which provides an on-the-fly flat view of a certain cell.
//! The presented view is flattened until leaf cells.
//! Internally this works by using component IDs that are actually paths through the hierarchy.

use crate::traits::{HierarchyBase, NetlistBase};
use crate::netlist::util::NetlistUtil;
use std::collections::{HashMap, HashSet};

/// Wrapper around a netlist which provides an on-the-fly flat view of a certain cell.
/// The presented view is flattened until leaf cells.
/// Internally this works by using component IDs that are actually paths through the hierarchy.
///
/// Names are constructed by creating concatenating the names of the path elements
/// with a separator string inbetween.
pub struct FlatView<'a, N> {
    /// Sequence used to separate path elements when creating qualified names.
    /// Names of the original netlist are not allowed to contain the path separator.
    path_separator: String,
    /// Underlying netlist data structure.
    base: &'a N,
}

impl<'a, N: HierarchyBase> HierarchyBase for FlatView<'a, N> {
    type NameType = N::NameType;
    type CellId = N::CellId;
    type CellInstId = Vec<N::CellInstId>;

    fn cell_by_name(&self, name: &str) -> Option<Self::CellId> {
        unimplemented!()
    }

    fn cell_instance_by_name(&self, parent_cell: &Self::CellId, name: &str) -> Option<Self::CellInstId> {
        unimplemented!()
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        self.base.cell_name(cell)
    }

    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        // Try to find the name of each path element.
        let path_names: Option<Vec<_>> = cell_inst.iter()
            .map(|inst| self.base.cell_instance_name(inst))
            .collect();
        // If a name could be found for each element
        // join them with the path separator.
        path_names.map(|names|
            names.join(&self.path_separator).into()
        )
    }

    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.base.parent_cell(&cell_instance[0])
    }

    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.base.template_cell(&cell_instance[cell_instance.len()-1])
    }

    fn for_each_cell<F>(&self, f: F) where F: FnMut(Self::CellId) -> () {
        self.base.for_each_cell(f)
    }

    fn for_each_cell_instance<F>(&self, cell: &Self::CellId, mut f: F) where F: FnMut(Self::CellInstId) -> () {

        // Depth-first traversal of the dependency graph.
        // Start with the top-level instances.
        let mut stack = vec![self.base.each_cell_instance(cell)];

        // Path through the hierarchy to the current cell.
        let mut path = vec![];

        // Work through all the levels until none is left.
        while let Some(mut insts) = stack.pop() {
            // Take the next instance from the current level...
            if let Some(inst) = insts.next() {
                // ... and directly push the current level again on the stack.
                stack.push(insts);
                let template = self.base.template_cell(&inst);
                path.push(inst);

                if self.base.num_child_instances(&template) == 0 {
                    // Leaf cell.
                    f(path.clone())
                } else {
                    // Push new level.
                    let sub_insts = self.base.each_cell_instance(&template);
                    stack.push(sub_insts);
                }
            } else {
                // insts is empty. We go a level up.
                path.pop();
            }
        }

    }

    fn for_each_cell_dependency<F>(&self, cell: &Self::CellId, mut f: F) where F: FnMut(Self::CellId) -> () {
        let mut visited = HashSet::new();
        let mut stack = self.base.each_cell_dependency_vec(cell);
        while let Some(dep) = stack.pop() {
            if !visited.contains(&dep) {
                // Find child dependencies.
                stack.extend(self.base.each_cell_dependency(&dep));
                // Visit the dependency.
                f(dep.clone());
                // Remember we visited this dependency already.
                visited.insert(dep);
            }
        }
    }

    fn for_each_dependent_cell<F>(&self, cell: &Self::CellId, mut f: F) where F: FnMut(Self::CellId) -> () {
        // Only top-level cells can be dependent cells in the flat view.
        let mut visited = HashSet::new();
        let mut stack = self.base.each_dependent_cell_vec(cell);
        while let Some(dep) = stack.pop() {
            if !visited.contains(&dep) {
                visited.insert(dep.clone());
                let is_top = self.base.num_dependent_cells(&dep) == 0;
                if is_top {
                    f(dep);
                } else {
                    // Follow towards the root.
                    stack.extend(self.base.each_dependent_cell(&dep));
                }
            }
        }
    }

    fn for_each_cell_reference<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        unimplemented!()
    }

    fn num_child_instances(&self, cell: &Self::CellId) -> usize {
        let num_non_flat_children = self.base.num_child_instances(cell);
        if num_non_flat_children == 0 {
            0
        } else {
            // Count how many times each cell is instantiated.
            let mut counted_cells: HashMap<N::CellId, usize> = Default::default();
            self.base.for_each_cell_instance(cell, |inst| {
                let template = self.base.template_cell(&inst);
                *counted_cells.entry(template)
                    .or_insert(0) += 1;
            });

            // Compute recursively the number of children.
            counted_cells.into_iter()
                .map(|(cell, num)| num * self.num_child_instances(&cell))
                .sum()
        }
    }

    fn num_cells(&self) -> usize {
        self.base.num_cells()
    }
}