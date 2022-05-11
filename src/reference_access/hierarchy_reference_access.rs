// Copyright (c) 2020-2021 Thomas Kramer.
// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::traits::HierarchyBase;

/// Trait that provides object-like read access to a cell hierarchy structure and its elements.
pub trait HierarchyReferenceAccess: HierarchyBase
{
    /// Iterate over all cell objects.
    fn each_cell_ref(&self) -> Box<dyn Iterator<Item=CellRef<Self>> + '_> {
        Box::new(self.each_cell()
            .map(move |id| self.cell_ref(&id))
        )
    }

    /// Get a cell object by its ID.
    fn cell_ref(&self, cell_id: &Self::CellId) -> CellRef<'_, Self> {
        CellRef {
            base: self,
            id: cell_id.clone(),
        }
    }

    /// Get a cell instance object by its ID.
    fn cell_instance_ref(&self, inst_id: &Self::CellInstId) -> CellInstRef<'_, Self> {
        CellInstRef {
            base: self,
            id: inst_id.clone(),
        }
    }
}

impl<T: HierarchyBase> HierarchyReferenceAccess for T {}

/// A reference to a cell.
/// This is just a wrapper around a netlist and a cell ID.
pub struct CellRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent data structure.
    pub(super) base: &'a H,
    /// ID of the corresponding cell.
    pub(super) id: H::CellId,
}

// impl<'a, H: HierarchyBase> AsRef<H> for CellRef<'a, H> {
//     fn as_ref(&self) -> &H {
//         self.base
//     }
// }
//
//
// impl<'a, H: HierarchyBase> Deref for CellRef<'a, H> {
//     type Target = H;
//
//     fn deref(&self) -> &Self::Target {
//         self.base
//     }
// }

impl<'a, H: HierarchyBase> Clone for CellRef<'a, H> {
    fn clone(&self) -> Self {
        Self {
            base: self.base,
            id: self.id.clone(),
        }
    }
}

impl<'a, H: HierarchyBase> CellRef<'a, H> {
    /// Access the base structure.
    pub fn base(&self) -> &'_ H {
        self.base
    }

    /// Get the ID of this cell.
    pub fn id(&self) -> H::CellId {
        self.id.clone()
    }

    /// Get the name of the cell.
    pub fn name(&self) -> H::NameType {
        self.base.cell_name(&self.id)
    }

    /// Iterate over the IDs of all child instances.
    pub fn each_cell_instance_id(&self) -> impl Iterator<Item=H::CellInstId> + '_ {
        self.base.each_cell_instance(&self.id)
    }

    /// Iterate over all child instances.
    pub fn each_cell_instance(&self) -> impl Iterator<Item=CellInstRef<'a, H>> + '_ {
        self.each_cell_instance_id()
            .map(move |id| CellInstRef {
                base: self.base,
                id,
            })
    }

    /// Find a child instance by its name.
    pub fn cell_instance_by_name(&self, name: &str) -> Option<CellInstRef<'a, H>> {
        self.base.cell_instance_by_name(&self.id, name)
            .map(|id| CellInstRef {
                base: self.base,
                id,
            })
    }

    /// Iterate over the IDs of all instances of this cell.
    pub fn each_reference_id(&self) -> impl Iterator<Item=H::CellInstId> + '_ {
        self.base.each_cell_reference(&self.id)
    }

    /// Iterate over the of all instances of this cell.
    pub fn each_reference(&self) -> impl Iterator<Item=CellInstRef<'a, H>> + '_ {
        self.each_reference_id()
            .map(move |id| CellInstRef {
                base: self.base,
                id,
            })
    }

    /// Get the total number of usages of this cell.
    pub fn num_references(&self) -> usize {
        self.base.num_cell_references(&self.id)
    }

    /// Iterate over all dependencies of this cell.
    pub fn each_cell_dependency(&self) -> impl Iterator<Item=CellRef<'a, H>> + '_ {
        self.base.each_cell_dependency(&self.id)
            .map(move |id| CellRef {
                base: self.base,
                id,
            })
    }

    /// Get the total number of direct dependencies of this cell.
    pub fn num_cell_dependencies(&self) -> usize {
        self.base.num_cell_dependencies(&self.id)
    }

    /// Iterate over all cells that directly depend on this cell.
    pub fn each_dependent_cell(&self) -> impl Iterator<Item=CellRef<'a, H>> + '_ {
        self.base.each_dependent_cell(&self.id)
            .map(move |id| CellRef {
                base: self.base,
                id,
            })
    }

    /// Get the total number of cells which depend on this cell (i.e. use it).
    pub fn num_dependent_cells(&self) -> usize {
        self.base.num_dependent_cells(&self.id)
    }

    /// Get the number of cell instances inside the `cell`.
    pub fn num_child_instances(&self) -> usize {
        self.base.num_child_instances(&self.id)
    }
}


/// Default implementation for `CellInstRef`.
/// This is just a wrapper around a netlist and a cell ID.
pub struct CellInstRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent netlist.
    pub(super) base: &'a H,
    /// ID of the corresponding cell instance.
    pub(super) id: H::CellInstId,
}


impl<'a, H: HierarchyBase> CellInstRef<'a, H> {
    /// Access the base structure.
    pub fn base(&self) -> &'_ H {
        self.base
    }

    /// Get the ID of this cell instance.
    pub fn id(&self) -> H::CellInstId {
        self.id.clone()
    }

    /// Get the name of the cell instance.
    pub fn name(&self) -> Option<H::NameType> {
        self.base.cell_instance_name(&self.id)
    }

    /// Get the parent cell of this instance.
    pub fn parent(&self) -> CellRef<'a, H> {
        CellRef {
            base: self.base,
            id: self.parent_id(),
        }
    }

    /// Get the template cell of this instance.
    pub fn template(&self) -> CellRef<'a, H> {
        CellRef {
            base: self.base,
            id: self.template_id(),
        }
    }

    /// Get the ID of the parent cell of this instance.
    pub fn parent_id(&self) -> H::CellId {
        self.base.parent_cell(&self.id)
    }

    /// Get the ID of the template cell of this instance.
    pub fn template_id(&self) -> H::CellId {
        self.base.template_cell(&self.id)
    }
}