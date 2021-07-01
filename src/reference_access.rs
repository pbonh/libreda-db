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
    fn cell(&self, cell_id: &Self::CellId) -> Box<dyn CellRef<H=Self> + '_> where Self: Sized {
        Box::new(
            DefaultCellRef {
                base_struct: self,
                id: cell_id.clone(),
            }
        )
    }

    /// Get a cell instance object by its ID.
    fn cell_instance(&self, inst_id: &Self::CellInstId) -> Box<dyn CellInstRef<H=Self> + '_> where Self: Sized {
        Box::new(
            DefaultCellInstRef {
                base_struct: self,
                id: inst_id.clone(),
            }
        )
    }
}

/// A reference to a cell.
pub trait CellRef {
    /// Base type of the hierarchical structure.
    type H: HierarchyBase;

    /// Get the ID of this cell.
    fn id(&self) -> <<Self as CellRef>::H as HierarchyBase>::CellId;

    /// Get the name of the cell.
    fn name(&self) -> <<Self as CellRef>::H as HierarchyBase>::NameType;

    /// Iterate over the IDs of all child instances.
    fn each_cell_instance_id(&self) -> Box<dyn Iterator<Item=<<Self as CellRef>::H as HierarchyBase>::CellInstId> + '_>;

    /// Iterate over the IDs of all instances of this cell.
    fn each_reference_id(&self) -> Box<dyn Iterator<Item=<<Self as CellRef>::H as HierarchyBase>::CellInstId> + '_>;
}

/// A reference to a cell instance.
pub trait CellInstRef {
    /// Base type of the hierarchical structure.
    type H: HierarchyBase;

    /// Get the ID of this cell.
    fn id(&self) -> <<Self as CellInstRef>::H as HierarchyBase>::CellInstId;

    /// Get the name of the cell.
    fn name(&self) -> Option<<<Self as CellInstRef>::H as HierarchyBase>::NameType>;

    /// Get the the parent cell.
    fn parent(&self) -> Box<dyn CellRef<H=Self::H> + '_>;

    /// Get the the template cell.
    fn template(&self) -> Box<dyn CellRef<H=Self::H> + '_>;

    /// Get the ID of the parent cell.
    fn parent_id(&self) -> <<Self as CellInstRef>::H as HierarchyBase>::CellId;

    /// Get the ID of the template cell.
    fn template_id(&self) -> <<Self as CellInstRef>::H as HierarchyBase>::CellId;
}
//
// /// Default implementation.
// pub struct DefaultReferenceAccess<'a, H: HierarchyBase + ?Sized> {
//     /// Reference to the parent data structure.
//     base_struct: &'a H,
// }
//
//
// /// Wrapper trait.
// pub trait DefaultReferenceAccessWrapper: HierarchyBase {
//
// }

/// Default implementation for `CellRef`.
/// This is just a wrapper around a netlist and a cell ID.
pub struct DefaultCellRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent data structure.
    base_struct: &'a H,
    /// ID of the corresponding cell.
    id: H::CellId,
}

impl<'a, H: HierarchyBase> CellRef for DefaultCellRef<'a, H> {
    type H = H;

    fn id(&self) -> H::CellId {
        self.id.clone()
    }

    fn name(&self) -> H::NameType {
        self.base_struct.cell_name(&self.id)
    }

    fn each_cell_instance_id(&self) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellInstId> + '_> {
        self.base_struct.each_cell_instance(&self.id)
    }

    fn each_reference_id(&self) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellInstId> + '_> {
        self.base_struct.each_cell_reference(&self.id)
    }
}


/// Default implementation for `CellInstRef`.
/// This is just a wrapper around a netlist and a cell ID.
pub struct DefaultCellInstRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent netlist.
    base_struct: &'a H,
    /// ID of the corresponding cell instance.
    id: H::CellInstId,
}

impl<'a, H: HierarchyBase> CellInstRef for DefaultCellInstRef<'a, H> {
    type H = H;

    fn id(&self) -> H::CellInstId {
        self.id.clone()
    }

    fn name(&self) -> Option<H::NameType> {
        self.base_struct.cell_instance_name(&self.id)
    }

    fn parent(&self) -> Box<dyn CellRef<H=Self::H> + '_> {
        Box::new(
            DefaultCellRef {
                base_struct: self.base_struct,
                id: self.parent_id(),
            }
        )
    }

    fn template(&self) -> Box<dyn CellRef<H=Self::H> + '_> {
        Box::new(
            DefaultCellRef {
                base_struct: self.base_struct,
                id: self.template_id(),
            }
        )
    }

    fn parent_id(&self) -> H::CellId {
        self.base_struct.parent_cell(&self.id)
    }

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