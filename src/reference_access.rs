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


//! # Experimental
//! Wrapper around the `HierarchyBase` trait. Provides more object like access methods.
//!

use crate::traits::HierarchyBase;

impl<T: HierarchyBase> HierarchyReferenceAccess for T {}

/// Trait that provides object-like read access to a hierarchical netlist or layout structure and its elements.
pub trait HierarchyReferenceAccess: HierarchyBase
{
    /// Get a cell object by its ID.
    fn cell(&self, cell_id: &Self::CellId) -> CellRef<'_, Self> {
        CellRef {
            base_struct: self,
            id: cell_id.clone(),
        }
    }

    /// Get a cell instance object by its ID.
    fn cell_instance(&self, inst_id: &Self::CellInstId) -> CellInstRef<'_, Self> {
        CellInstRef {
            base_struct: self,
            id: inst_id.clone(),
        }
    }
}


/// A reference to a cell.
/// This is just a wrapper around a netlist and a cell ID.
pub struct CellRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent data structure.
    base_struct: &'a H,
    /// ID of the corresponding cell.
    id: H::CellId,
}

impl<'a, H: HierarchyBase> CellRef<'a, H> {
    /// Get the ID of this cell.
    pub fn id(&self) -> H::CellId {
        self.id.clone()
    }

    /// Get the name of the cell.
    pub fn name(&self) -> H::NameType {
        self.base_struct.cell_name(&self.id)
    }

    /// Iterate over the IDs of all child instances.
    pub fn each_cell_instance_id(&self) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
        self.base_struct.each_cell_instance(&self.id)
    }

    /// Iterate over the IDs of all instances of this cell.
    pub fn each_reference_id(&self) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
        self.base_struct.each_cell_reference(&self.id)
    }
}


/// Default implementation for `CellInstRef`.
/// This is just a wrapper around a netlist and a cell ID.
pub struct CellInstRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent netlist.
    base_struct: &'a H,
    /// ID of the corresponding cell instance.
    id: H::CellInstId,
}

impl<'a, H: HierarchyBase> CellInstRef<'a, H> {
    /// Get the ID of this cell instance.
    fn id(&self) -> H::CellInstId {
        self.id.clone()
    }

    /// Get the name of the cell instance.
    fn name(&self) -> Option<H::NameType> {
        self.base_struct.cell_instance_name(&self.id)
    }

    /// Get the parent cell of this instance.
    fn parent(&self) -> CellRef<'_, H> {
            CellRef {
                base_struct: self.base_struct,
                id: self.parent_id(),
            }
    }

    /// Get the template cell of this instance.
    fn template(&self) -> CellRef<'_, H> {
        CellRef {
            base_struct: self.base_struct,
            id: self.template_id(),
        }
    }

    /// Get the ID of the parent cell of this instance.
    fn parent_id(&self) -> H::CellId {
        self.base_struct.parent_cell(&self.id)
    }

    /// Get the ID of the template cell of this instance.
    fn template_id(&self) -> H::CellId {
        self.base_struct.template_cell(&self.id)
    }
}


#[test]
fn test_chip_reference_access() {
    use crate::prelude::*;
    use crate::chip::Chip;

    let mut chip = Chip::new();
    let top = chip.create_cell("TOP".into());
    let sub = chip.create_cell("SUB".into());
    let sub_inst1 = chip.create_cell_instance(&top, &sub, Some("inst1".into()));

    let top_ref = chip.cell(&top);
    assert_eq!(&top_ref.id(), &top);

    let sub_inst1_ref = chip.cell_instance(&sub_inst1);
    assert_eq!(&sub_inst1_ref.id(), &sub_inst1);
    assert_eq!(sub_inst1_ref.parent().id(), top_ref.id());
    assert_eq!(&sub_inst1_ref.template().id(), &sub);
}