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

#![allow(missing_docs)]

use crate::traits::{HierarchyBase, HierarchyEdit};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, Arc};

/// Wrapper around a Netlist or Layout, etc. into a `Arc<RwLock<_>>` to provide
/// save read and write access.
pub struct Base<T> {
    base: Arc<RwLock<T>>
}

impl<T> Base<T> {
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

impl<T> Clone for Base<T> {
    fn clone(&self) -> Self {
        Self { base: self.base.clone() }
    }
}


impl<H: HierarchyBase> Base<H> {

    fn cell(&self, id: H::CellId) -> CellRef<H> {
        CellRef {
            base: self.clone(),
            id
        }
    }

    fn cell_inst(&self, id: H::CellInstId) -> CellInstRef<H> {
        CellInstRef {
            base: self.clone(),
            id
        }
    }


    pub fn each_cell(&self) -> Vec<CellRef<H>> {
        self.read()
            .each_cell()
            .map(|id| self.cell(id))
            .collect()
    }

    pub fn cell_by_name(&self, name: &str) -> Option<CellRef<H>> {
        self.read()
            .cell_by_name(name)
            .map(|id| self.cell(id))
    }

}


impl<H: HierarchyEdit> Base<H> {

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
    pub(super) base: Base<H>,
    /// ID of the corresponding cell.
    pub(super) id: H::CellId,
}

impl<H: HierarchyBase> CellRef<H> {
    /// Access the base structure.
    pub fn base(&self) -> &Base<H> {
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
pub struct CellInstRef<H: HierarchyBase> {
    /// Reference to the parent netlist.
    pub(super) base: Base<H>,
    /// ID of the corresponding cell instance.
    pub(super) id: H::CellInstId,
}


impl<H: HierarchyBase> CellInstRef<H> {
    /// Access the base structure.
    pub fn base(&self) -> &Base<H> {
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