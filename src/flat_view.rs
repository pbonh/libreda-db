/*
 * Copyright (c) 2020-2021 Thomas Kramer.
 *
 * This file is part of LibrEDA
 * (see https://codeberg.org/libreda/arboreus-db).
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

use crate::traits::{HierarchyBase};
use std::collections::{HashMap, HashSet};

/// Wrapper around ID types.
/// This wrapper makes sure that the flat view uses other ID types than the
/// underlying hierarchical view.
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct FlatId<T>(T);

/// Wrapper around a netlist which provides an on-the-fly flat view of a certain cell.
/// The presented view is flattened until leaf cells.
/// Internally this works by using component IDs that are actually paths through the hierarchy.
///
/// Names are constructed by concatenating the names of the path elements
/// with a separator string in between.
///
/// # Example
///
/// ```
/// use libreda_db::prelude::{Chip, HierarchyBase, HierarchyEdit, FlatView};
///
/// // Create a simple hierarchy.
/// let mut chip = Chip::new();
/// let top = chip.create_cell("TOP".into());
/// let intermediate = chip.create_cell("INTERMEDIATE".into());
/// let leaf1 = chip.create_cell("LEAF1".into());
/// let leaf2 = chip.create_cell("LEAF2".into());
///
/// // The intermediate cell contains two instances of leaf1 and one instance of leaf2.
/// chip.create_cell_instance(&intermediate, &leaf1, Some("leaf1_inst1".into()));
/// chip.create_cell_instance(&intermediate, &leaf1, Some("leaf1_inst2".into()));
/// chip.create_cell_instance(&intermediate, &leaf2, Some("leaf2_inst1".into()));
///
/// // Create two instances of the intermediate cell in the TOP cell.
/// chip.create_cell_instance(&top, &intermediate, Some("intermediate1".into()));
/// chip.create_cell_instance(&top, &intermediate, Some("intermediate2".into()));
///
/// // Create the flat view.
///
/// let flat = FlatView::new_with_separator(&chip, ":".to_string());
/// let flat_top = flat.cell_by_name("TOP").expect("TOP not found in flat view.");
/// // There are 2 instances of the intermediate cell which contains 3 leaf cells,
/// // so now the flattened top should contain 2*3 instances.
/// assert_eq!(flat.num_child_instances(&flat_top), 2*3);
///
/// // Get a cell instance with the path string.
/// let inst = flat.cell_instance_by_name(&flat_top, "intermediate1:leaf1_inst1").expect("Instance not found.");
/// // Instance names are assembled from the path.
/// assert_eq!(flat.cell_instance_name(&inst).unwrap().as_str(), "intermediate1:leaf1_inst1");
///
/// // There should be 4 instances of the LEAF1 cell now.
/// assert_eq!(flat.each_cell_reference(&leaf1).count(), 2*2);
/// ```
pub struct FlatView<'a, N> {
    /// Sequence used to separate path elements when creating qualified names.
    /// Names of the original netlist are not allowed to contain the path separator.
    path_separator: String,
    /// Underlying netlist data structure.
    base: &'a N,
}

impl<'a, N: HierarchyBase> FlatView<'a, N> {
    /// Create a new flat view of `base`.
    /// Use "/" as a path separator in names.
    pub fn new(base: &'a N) -> Self {
        Self {
            path_separator: "/".to_string(),
            base,
        }
    }

    /// Create a new flat view of `base`.
    /// Use a custom path separator in concatenated names.
    pub fn new_with_separator(base: &'a N, path_separator: String) -> Self {
        Self {
            path_separator,
            base,
        }
    }

    fn cell_is_leaf(&self, cell: &N::CellId) -> bool {
        self.base.num_child_instances(&cell) == 0
    }

    fn cell_exists_in_flat_view(&self, cell: &N::CellId) -> bool {
        !self.cell_is_flattened(cell)
    }

    /// Check if the cell got flattened and does not
    /// exist in the flat view.
    fn cell_is_flattened(&self, cell: &N::CellId) -> bool {
        let is_top = self.base.num_dependent_cells(&cell) == 0;
        let is_leaf = self.cell_is_leaf(cell);
        !is_top && !is_leaf
    }
}


impl<'a, N: HierarchyBase> HierarchyBase for FlatView<'a, N> {
    type NameType = N::NameType;
    type CellId = N::CellId;
    type CellInstId = Vec<N::CellInstId>;

    fn cell_by_name(&self, name: &str) -> Option<Self::CellId> {
        let cell = self.base.cell_by_name(name);
        if let Some(cell) = cell {
            if self.cell_exists_in_flat_view(&cell) {
                Some(cell)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn cell_instance_by_name(&self, parent_cell: &Self::CellId, name: &str) -> Option<Self::CellInstId> {
        let path = name.split(&self.path_separator);
        let mut parent_cell = parent_cell.clone();
        let mut current_inst = vec![];
        // Resolve the path.
        // For each path element...
        for name in path {
            // Find the child in the current parent.
            let inst = self.base.cell_instance_by_name(&parent_cell, name);
            if let Some(inst) = inst {
                // Descend into the child.
                parent_cell = self.base.template_cell(&inst);
                current_inst.push(inst);
            } else {
                // No child could be found.
                current_inst.clear();
                break;
            }
        }
        if current_inst.is_empty() {
            None
        } else {
            Some(current_inst)
        }
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        let name = self.base.cell_name(cell);

        if self.cell_is_flattened(cell) {
            panic!("Cell does not exist in flat view: {}", &name);
        }
        name
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
        self.base.template_cell(&cell_instance[cell_instance.len() - 1])
    }

    fn for_each_cell<F>(&self, mut f: F) where F: FnMut(Self::CellId) -> () {
        self.base.for_each_cell(|c| {
            // Iterate over top-level and leaf cells only.
            if self.cell_exists_in_flat_view(&c) {
                f(c);
            }
        })
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
                if self.cell_exists_in_flat_view(&dep) {
                    f(dep.clone());
                }
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
                if self.cell_exists_in_flat_view(&dep) {
                    f(dep);
                } else {
                    // Follow towards the root.
                    stack.extend(self.base.each_dependent_cell(&dep));
                }
            }
        }
    }

    fn for_each_cell_reference<F>(&self, cell: &Self::CellId, mut f: F)
        where F: FnMut(Self::CellInstId) -> () {
        assert!(self.cell_exists_in_flat_view(&cell), "Cell does not exist in flat view: {}", self.base.cell_name(cell));

        let mut references = vec![self.base.each_cell_reference(&cell)];
        let mut path_rev = vec![];

        while let Some(mut refs) = references.pop() {
            if let Some(r) = refs.next() {
                references.push(refs);
                let parent = self.base.parent_cell(&r);
                path_rev.push(r.clone());
                if self.cell_exists_in_flat_view(&parent) {
                    // Reached the top.
                    let mut path = path_rev.clone();
                    path.reverse();
                    f(path);
                } else {
                    // Get parent references.
                    references.push(self.base.each_cell_reference(&parent));
                }
            } else {
                // Worked through all references on this level.
                path_rev.pop();
            }
        }
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
                .map(|(cell, num)| num * self.base.num_child_instances(&cell))
                .sum()
        }
    }

    fn num_cells(&self) -> usize {
        let mut count = 0;
        self.for_each_cell(|_| count += 1);
        count
    }
}

// On-the-fly flattening of nets is not solved yet.
// The main difficulty is: Many nets might now be fused together into one net. How can this be uniquely and efficiently represented?
// impl<'a, N: NetlistBase> NetlistBase for FlatView<'a, N> {
//     type PinId = N::PinId;
//     type PinInstId = (Self::CellInstId, N::PinInstId);
//     // Pin instances need to be extended with the path through the hierarhcy.
//     type NetId = (Self::CellInstId, N::NetId);
//
//     fn template_pin(&self, (_, pin_instance): &Self::PinInstId) -> Self::PinId {
//         self.base.template_pin(pin_instance)
//     }
//
//     fn pin_direction(&self, pin: &Self::PinId) -> Direction {
//         self.base.pin_direction(pin)
//     }
//
//     fn pin_name(&self, pin: &Self::PinId) -> Self::NameType {
//         self.base.pin_name(pin)
//     }
//
//     fn pin_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::PinId> {
//         self.base.pin_by_name(parent_circuit, name)
//     }
//
//     fn parent_cell_of_pin(&self, pin: &Self::PinId) -> Self::CellId {
//         self.base.parent_cell_of_pin(pin)
//     }
//
//     fn parent_of_pin_instance(&self, (cell_inst, _pin_inst): &Self::PinInstId) -> Self::CellInstId {
//         cell_inst.clone()
//     }
//
//     fn parent_cell_of_net(&self, (path, net): &Self::NetId) -> Self::CellId {
//         if let Some(instance) = path.iter().nth(0) {
//             // The parent of the flattened net is equal to the parent of the first
//             // cell instance in the path.
//             self.base.parent_cell(instance)
//         } else {
//             // The net lives in the top-cell.
//             self.base.parent_cell_of_net(net)
//         }
//     }
//
//     fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId> {
//         let net = self.base.net_of_pin(pin);
//         net.map(
//             |n| (vec![], n)
//         )
//     }
//
//     fn net_of_pin_instance(&self, (path, pin_instance): &Self::PinInstId) -> Option<Self::NetId> {
//         let non_flattened_net = self.base.net_of_pin_instance(pin_instance);
//         non_flattened_net.map(
//             |n| (path.clone(), n)
//         )
//     }
//
//     fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId {
//         (vec![], self.base.net_zero(parent_circuit))
//     }
//
//     fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId {
//         (vec![], self.base.net_one(parent_circuit))
//     }
//
//     fn net_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::NetId> {
//         // Find last path separator after which comes the net name.
//
//         if let Some(last_separator_pos) = name.rfind(self.path_separator.as_str()) {
//             let path_string = &name[0..last_separator_pos];
//             let net_name = &name[last_separator_pos + 1..name.len()];
//
//             // Resolve cell instance.
//             if let Some(cell_inst) = self.cell_instance_by_name(parent_circuit, path_string) {
//                 let template = self.base.template_cell(&cell_inst[cell_inst.len() - 1]);
//                 let net = self.base.net_by_name(&template, net_name);
//                 net.map(
//                     |n| (cell_inst, n)
//                 )
//             } else {
//                 // Cell instance not found.
//                 None
//             }
//         } else {
//             // No separator in net name. Look directly in the top cell.
//             let net = self.base.net_by_name(parent_circuit, name);
//             net.map(
//                 |n| (vec![], n)
//             )
//         }
//     }
//
//     fn net_name(&self, (path, net): &Self::NetId) -> Option<Self::NameType> {
//         if let Some(net_name) = self.base.net_name(net) {
//             // Try to find the name of each path element.
//             let path_names: Option<Vec<_>> = path.iter()
//                 .map(|inst| self.base.cell_instance_name(inst))
//                 .collect();
//             // If a name could be found for each element
//             // join them with the path separator.
//             path_names.map(|mut names| {
//                 names.push(net_name);
//                 names.join(&self.path_separator).into()
//             })
//         } else {
//             None
//         }
//     }
//
//     fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> () {
//         self.base.for_each_pin(circuit, f)
//     }
//
//     fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CellInstId, mut f: F) where F: FnMut(Self::PinInstId) -> () {
//         let hierarchical_inst = &circuit_inst[circuit_inst.len() - 1];
//         self.base.for_each_pin_instance(hierarchical_inst, |p| {
//             f((circuit_inst.clone(), p))
//         })
//     }
//
//     fn for_each_internal_net<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::NetId) -> () {
//         unimplemented!()
//     }
//
//     fn num_pins(&self, cell: &Self::CellId) -> usize {
//         self.base.num_pins(cell)
//     }
//
//     fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> () {
//         unimplemented!()
//     }
//
//     fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> () {
//         unimplemented!()
//     }
//
//     fn num_internal_nets(&self, parent: &Self::CellId) -> usize {
//         unimplemented!()
//     }
// }
//
// #[cfg(test)]
// mod tests_with_hierarchy {
//     use crate::prelude::Chip;
//     use crate::prelude::*;
//     use crate::flat_view::FlatView;
//
//     fn create_test_chip() -> Chip {
//         let mut chip = Chip::new();
//         let top1 = chip.create_cell("TOP1".into());
//         let top2 = chip.create_cell("TOP2".into());
//         let intermediate = chip.create_cell("INTERMEDIATE".into());
//         let leaf1 = chip.create_cell("LEAF1".into());
//         let leaf2 = chip.create_cell("LEAF2".into());
//
//         chip.create_cell_instance(&intermediate, &leaf1, Some("leaf1_inst1".into()));
//         chip.create_cell_instance(&intermediate, &leaf1, Some("leaf1_inst2".into()));
//         chip.create_cell_instance(&intermediate, &leaf2, Some("leaf2_inst1".into()));
//         chip.create_cell_instance(&intermediate, &leaf2, Some("leaf2_inst2".into()));
//
//         chip.create_cell_instance(&top1, &intermediate, Some("intermediate_inst1".into()));
//         chip.create_cell_instance(&top1, &intermediate, Some("intermediate_inst2".into()));
//
//         // Create instances inanother cell with same names as in TOP1.
//         chip.create_cell_instance(&top2, &leaf1, Some("leaf1_inst1".into()));
//         chip.create_cell_instance(&top2, &leaf2, Some("leaf2_inst1".into()));
//         chip.create_cell_instance(&top2, &leaf2, Some("leaf2_inst2".into()));
//         chip
//     }
//
//     #[test]
//     fn test_num_cells() {
//         let chip = create_test_chip();
//         let flatview = FlatView::new(&chip);
//         assert_eq!(flatview.num_cells(), 4); // Two top cells, two leaf cells.
//     }
//
//     #[test]
//     fn test_access_top_cell() {
//         let chip = create_test_chip();
//
//         let flatview = FlatView::new(&chip);
//         let top1 = flatview.cell_by_name("TOP1").expect("Cell not found.");
//         assert_eq!(flatview.num_child_instances(&top1), 2 * 4);
//         assert_eq!(flatview.num_dependent_cells(&top1), 0);
//         assert_eq!(flatview.num_cell_dependencies(&top1), 2);
//         assert_eq!(flatview.each_cell_instance(&top1).count(), 8);
//     }
//
//     #[test]
//     fn test_find_template_cell() {
//         let chip = create_test_chip();
//         let flatview = FlatView::new(&chip);
//         let top1 = flatview.cell_by_name("TOP1").expect("Cell not found.");
//         let leaf1 = flatview.cell_by_name("LEAF1").expect("Cell not found.");
//
//         // Template
//         assert_eq!(
//             &flatview.template_cell(
//                 &flatview.cell_instance_by_name(&top1, "intermediate_inst1/leaf1_inst1",
//                 ).unwrap()),
//             &leaf1);
//     }
//
//     #[test]
//     fn test_find_instance_by_name() {
//         let chip = create_test_chip();
//         let flatview = FlatView::new(&chip);
//         let top1 = flatview.cell_by_name("TOP1").expect("Cell not found.");
//
//         // Find by name.
//         {
//             let names = vec![
//                 "intermediate_inst1/leaf1_inst1",
//                 "intermediate_inst2/leaf1_inst1",
//                 "intermediate_inst2/leaf2_inst1",
//                 "intermediate_inst2/leaf2_inst2",
//             ];
//             for name in names {
//                 let inst = flatview.cell_instance_by_name(&top1, name)
//                     .expect("instance not found");
//                 assert_eq!(flatview.cell_instance_name(&inst), Some(name.into()));
//
//                 // Parent
//                 assert_eq!(&flatview.parent_cell(&inst), &top1);
//             }
//         }
//     }
//
//     #[test]
//     fn test_count_references() {
//         let chip = create_test_chip();
//         let flatview = FlatView::new(&chip);
//         let top1 = flatview.cell_by_name("TOP1").expect("Cell not found.");
//         let leaf1 = flatview.cell_by_name("LEAF1").expect("Cell not found.");
//         let leaf2 = flatview.cell_by_name("LEAF2").expect("Cell not found.");
//
//         // References.
//         assert_eq!(flatview.num_cell_references(&leaf1), 2 * 2 + 1);
//         assert_eq!(flatview.num_cell_references(&leaf2), 2 * 2 + 2);
//         assert_eq!(flatview.num_cell_references(&top1), 0);
//     }
//
//
//     #[test]
//     fn test_another_top_cell() {
//         // TOP2 contains instances with same name as in TOP1.
//         let chip = create_test_chip();
//         let flatview = FlatView::new(&chip);
//         let top2 = flatview.cell_by_name("TOP2").expect("Cell not found.");
//
//         assert_eq!(flatview.num_dependent_cells(&top2), 0);
//         assert_eq!(flatview.num_cell_dependencies(&top2), 2);
//         assert_eq!(flatview.each_cell_instance(&top2).count(), 3);
//     }
//
// }
//
//
// #[cfg(test)]
// mod tests_with_netlist {
//     use crate::prelude::Chip;
//     use crate::prelude::*;
//     use crate::flat_view::FlatView;
//
//     fn create_test_netlist() -> Chip {
//         let mut chip = Chip::new();
//         let top = chip.create_cell("TOP".into());
//         let sub = chip.create_cell("SUB".into());
//         chip.create_net(&sub, Some("A".into()));
//
//         let inst_sub1 = chip.create_cell_instance(&top, &sub, Some("sub1".into()));
//         let inst_sub2 = chip.create_cell_instance(&top, &sub, Some("sub2".into()));
//
//         chip
//     }
//
//     #[test]
//     fn test_net_by_name() {
//         let chip = create_test_netlist();
//         let flatview = FlatView::new(&chip);
//         let top = flatview.cell_by_name("TOP").expect("Cell not found.");
//         assert!(flatview.net_by_name(&top, "sub1/A").is_some());
//         assert!(flatview.net_by_name(&top, "sub2/A").is_some());
//     }
//
//
// }