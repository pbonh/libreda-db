// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(missing_docs)]

use crate::traits::{HierarchyBase, HierarchyEdit};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, Arc};
use std::hash::{Hash, Hasher};

/// Wrapper around a Netlist or Layout, etc. into a `Arc<RwLock<_>>` to provide
/// save read and write access. Access is checked at runtime.
/// If only read access is required, then it might be more efficient to use [`crate::reference_access`].
///
/// In contrast to the API of [`HierarchyBase`] and others the object-like API avoids returning iterators
/// but returns vectors of elements. This allows to keep the lock-time short.
pub struct RwRefAccess<T> {
    base: Arc<RwLock<T>>
}

impl<T> RwRefAccess<T> {
    pub fn new(base: T) -> Self {
        Self { base: Arc::new(RwLock::new(base)) }
    }

    /// Get read access to the underlying data structure.
    ///
    /// # Panics
    /// Panics when called during an ongoing write access.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.base
            .read()
            .expect("Failed to get read access.")
    }

    /// Get exclusive write access to the underlying data structure.
    ///
    /// # Panics
    /// Panics when called during an ongoing read or write access.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.base
            .write()
            .expect("Failed to get write access.")
    }
}

impl<T> Clone for RwRefAccess<T> {
    fn clone(&self) -> Self {
        Self { base: self.base.clone() }
    }
}

impl<T> Eq for RwRefAccess<T> {}

impl<T> PartialEq for RwRefAccess<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.base, &other.base)
    }
}

impl<T> Hash for RwRefAccess<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.base).hash(state)
    }
}

impl<H: HierarchyBase> RwRefAccess<H> {
    /// Get a cell 'object' by its ID.
    fn cell(&self, id: H::CellId) -> CellRef<H> {
        CellRef {
            base: self.clone(),
            id,
        }
    }

    /// Get a cell instance 'object' by its ID.
    fn cell_inst(&self, id: H::CellInstId) -> CellInstRef<H> {
        CellInstRef {
            base: self.clone(),
            id,
        }
    }

    /// Get a vector with all cells.
    pub fn each_cell(&self) -> Vec<CellRef<H>> {
        self.read()
            .each_cell()
            .map(|id| self.cell(id))
            .collect()
    }

    /// Get the number of cells.
    pub fn num_cells(&self) -> usize {
        self.read().num_cells()
    }

    /// Find a cell by its name.
    pub fn cell_by_name(&self, name: &str) -> Option<CellRef<H>> {
        self.read()
            .cell_by_name(name)
            .map(|id| self.cell(id))
    }
}


impl<H: HierarchyEdit> RwRefAccess<H> {
    pub fn create_cell(&self, name: H::NameType) -> CellRef<H> {
        let id = self.write()
            .create_cell(name);
        self.cell(id)
    }

    pub fn remove_cell(&self, cell: CellRef<H>) {
        self.write()
            .remove_cell(&cell.id)
    }
}

/// A reference to a cell.
/// This is just a wrapper around a netlist and a cell ID.
#[derive(Clone)]
pub struct CellRef<H: HierarchyBase> {
    /// Reference to the parent data structure.
    pub(super) base: RwRefAccess<H>,
    /// ID of the corresponding cell.
    pub(super) id: H::CellId,
}

impl<T: HierarchyBase> Eq for CellRef<T> {}

impl<T: HierarchyBase> PartialEq for CellRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.id == other.id
    }
}

impl<T: HierarchyBase> Hash for CellRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.id.hash(state);
    }
}


impl<H: HierarchyBase> CellRef<H> {
    /// Access the base structure.
    pub fn base(&self) -> &RwRefAccess<H> {
        &self.base
    }

    /// Get the ID of this cell.
    pub fn id(&self) -> H::CellId {
        self.id.clone()
    }

    /// Get the name of the cell.
    pub fn name(&self) -> H::NameType {
        self.base.read().cell_name(&self.id)
    }

    /// Get the IDs of all cell instances in this cell.
    pub fn each_cell_instance_id(&self) -> Vec<H::CellInstId> {
        self.base.read().each_cell_instance_vec(&self.id)
    }

    /// Get all cell instances inside this cell.
    pub fn each_cell_instance(&self) -> Vec<CellInstRef<H>> {
        self.base.read()
            .each_cell_instance(&self.id)
            .map(move |id| CellInstRef {
                base: self.base.clone(),
                id,
            })
            .collect()
    }

    /// Find a child instance by its name.
    pub fn cell_instance_by_name(&self, name: &str) -> Option<CellInstRef<H>> {
        self.base.read().cell_instance_by_name(&self.id, name)
            .map(|id| CellInstRef {
                base: self.base.clone(),
                id,
            })
    }

    /// Get the IDs of all instances of this cell.
    pub fn each_reference_id(&self) -> Vec<H::CellInstId> {
        self.base.read().each_cell_reference_vec(&self.id)
    }

    /// Get all instances of this cell.
    pub fn each_reference(&self) -> Vec<CellInstRef<H>> {
        self.base.read()
            .each_cell_reference(&self.id)
            .map(|id| CellInstRef {
                base: self.base.clone(),
                id,
            })
            .collect()
    }

    /// Get all dependencies of this cell.
    pub fn each_cell_dependency(&self) -> Vec<CellRef<H>> {
        self.base.read()
            .each_cell_dependency(&self.id)
            .map(|id| CellRef {
                base: self.base.clone(),
                id,
            })
            .collect()
    }

    /// Get all cells that directly depend on this cell.
    pub fn each_dependent_cell(&self) -> Vec<CellRef<H>> {
        self.base.read()
            .each_dependent_cell(&self.id)
            .map(|id| CellRef {
                base: self.base.clone(),
                id,
            })
            .collect()
    }

    /// Get the number of cell instances inside the `cell`.
    pub fn num_child_instances(&self) -> usize {
        self.base.read().num_child_instances(&self.id)
    }
}


impl<H: HierarchyEdit> CellRef<H> {
    pub fn create_instance(&self, template: &CellRef<H>, name: Option<H::NameType>) -> CellInstRef<H> {
        let id = self.base.write()
            .create_cell_instance(&self.id, &template.id, name);
        self.base.cell_inst(id)
    }

    pub fn remove_instance(&self, inst: CellInstRef<H>) {
        self.base.write()
            .remove_cell_instance(&inst.id);
    }
}


/// Default implementation for `CellInstRef`.
/// This is just a wrapper around a netlist and a cell ID.
#[derive(Clone, Hash)]
pub struct CellInstRef<H: HierarchyBase> {
    /// Reference to the parent netlist.
    pub(super) base: RwRefAccess<H>,
    /// ID of the corresponding cell instance.
    pub(super) id: H::CellInstId,
}

impl<T: HierarchyBase> Eq for CellInstRef<T> {}

impl<T: HierarchyBase> PartialEq for CellInstRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.id == other.id
    }
}


impl<H: HierarchyBase> CellInstRef<H> {
    /// Access the base structure.
    pub fn base(&self) -> &RwRefAccess<H> {
        &self.base
    }

    /// Get the ID of this cell instance.
    pub fn id(&self) -> H::CellInstId {
        self.id.clone()
    }

    /// Get the name of the cell instance.
    pub fn name(&self) -> Option<H::NameType> {
        self.base.read().cell_instance_name(&self.id)
    }

    /// Get the parent cell of this instance.
    pub fn parent(&self) -> CellRef<H> {
        CellRef {
            base: self.base.clone(),
            id: self.parent_id(),
        }
    }

    /// Get the template cell of this instance.
    pub fn template(&self) -> CellRef<H> {
        CellRef {
            base: self.base.clone(),
            id: self.template_id(),
        }
    }

    /// Get the ID of the parent cell of this instance.
    pub fn parent_id(&self) -> H::CellId {
        self.base.read().parent_cell(&self.id)
    }

    /// Get the ID of the template cell of this instance.
    pub fn template_id(&self) -> H::CellId {
        self.base.read().template_cell(&self.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::chip::Chip;
    use crate::prelude::*;
    use crate::rw_reference_access::RwRefAccess;
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    /// Create a chip with two cells TOP and SUB. TOP contains an instance of SUB.
    fn create_test_chip() -> RwRefAccess<Chip> {
        let mut chip = Chip::new();
        let top = chip.create_cell("TOP".into());
        let sub = chip.create_cell("SUB".into());
        let _inst1 = chip.create_cell_instance(&top, &sub, Some("inst1".into()));
        RwRefAccess::new(chip)
    }

    #[test]
    fn create_rw_refaccess_from_mutable_reference() {
        let mut chip = Chip::new();
        let _rw_chip = RwRefAccess::new(&mut chip);
        // rw_chip.each_cell();
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_find_cell_by_name() {
        let chip = create_test_chip();
        let _top = chip.cell_by_name("TOP").unwrap();
        let _sub = chip.cell_by_name("SUB").unwrap();
    }

    #[test]
    fn test_find_cell_instance_by_name() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let _inst1 = top.cell_instance_by_name("inst1").unwrap();
    }

    #[test]
    fn test_cell_equality() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let sub = chip.cell_by_name("SUB").unwrap();
        assert!(top == top.clone());
        assert!(top != sub);
    }


    #[test]
    fn test_cell_hash() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let sub = chip.cell_by_name("SUB").unwrap();
        assert_eq!(hash(&top), hash(&top.clone()));
        assert_eq!(hash(&sub), hash(&sub.clone()));
        assert_ne!(hash(&top), hash(&sub)); // This assertion is expected to fail with very small probability.
    }

    #[test]
    fn test_cell_instance_equality() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let sub = chip.cell_by_name("SUB").unwrap();
        let inst1 = top.cell_instance_by_name("inst1").unwrap();
        let inst2 = top.create_instance(&sub, Some("inst2".into()));
        assert!(inst1 == inst1.clone());
        assert!(inst1 != inst2);
    }

    #[test]
    fn test_create_cell() {
        let chip = create_test_chip();
        chip.create_cell("NEW".into());
        chip.cell_by_name("NEW").unwrap();
    }

    #[test]
    fn test_create_instance() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let sub = chip.cell_by_name("SUB").unwrap();
        let inst = top.create_instance(&sub, Some("inst2".into()));
        assert!(inst.template() == sub);
        assert!(inst.parent() == top);
    }

    #[test]
    fn test_access_child_instances() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let children = top.each_cell_instance();
        assert_eq!(children.len(), 1);
    }


    #[test]
    fn test_cell_dependencies() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let sub = chip.cell_by_name("SUB").unwrap();
        assert_eq!(top.each_cell_dependency().len(), 1);
        assert_eq!(sub.each_cell_dependency().len(), 0);
    }

    #[test]
    fn test_dependent_cells() {
        let chip = create_test_chip();
        let top = chip.cell_by_name("TOP").unwrap();
        let sub = chip.cell_by_name("SUB").unwrap();
        assert_eq!(top.each_dependent_cell().len(), 0);
        assert_eq!(sub.each_dependent_cell().len(), 1);
    }
}