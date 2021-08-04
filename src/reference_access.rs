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

use crate::traits::{HierarchyBase, NetlistBase};

/// Trait that provides object-like read access to a cell hierarchy structure and its elements.
pub trait HierarchyReferenceAccess: HierarchyBase
{
    /// Get a cell object by its ID.
    fn cell(&self, cell_id: &Self::CellId) -> CellRef<'_, Self> {
        CellRef {
            base: self,
            id: cell_id.clone(),
        }
    }

    /// Get a cell instance object by its ID.
    fn cell_instance(&self, inst_id: &Self::CellInstId) -> CellInstRef<'_, Self> {
        CellInstRef {
            base: self,
            id: inst_id.clone(),
        }
    }
}

impl<T: HierarchyBase> HierarchyReferenceAccess for T {}

/// Trait that provides object-like read access to a hierarchical netlist structure and its elements.
pub trait NetlistReferenceAccess: NetlistBase {
    /// Get a reference to a pin.
    fn pin(&self, pin: &Self::PinId) -> PinRef<'_, Self> {
        PinRef {
            base: self,
            id: pin.clone(),
        }
    }

    /// Get a reference to a pin instance.
    fn pin_instance(&self, id: &Self::PinInstId) -> PinInstRef<'_, Self> {
        PinInstRef {
            base: self,
            id: id.clone(),
        }
    }

    /// Get a reference to a net.
    fn net(&self, net: &Self::NetId) -> NetRef<'_, Self> {
        NetRef {
            base: self,
            id: net.clone(),
        }
    }
}

impl<T: NetlistBase> NetlistReferenceAccess for T {}

/// A reference to a cell.
/// This is just a wrapper around a netlist and a cell ID.
pub struct CellRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent data structure.
    base: &'a H,
    /// ID of the corresponding cell.
    id: H::CellId,
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
    pub fn each_cell_instance_id(&self) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
        self.base.each_cell_instance(&self.id)
    }

    /// Iterate over all child instances.
    pub fn each_cell_instance(&self) -> Box<dyn Iterator<Item=CellInstRef<'a, H>> + '_> {
        Box::new(self.each_cell_instance_id()
            .map(move |id| CellInstRef {
                base: self.base,
                id,
            }))
    }

    /// Iterate over the IDs of all instances of this cell.
    pub fn each_reference_id(&self) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
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

    /// Iterate over all dependencies of this cell.
    pub fn each_cell_dependency(&self) -> impl Iterator<Item=CellRef<'a, H>> + '_ {
        self.base.each_cell_dependency(&self.id)
            .map(move |id| CellRef {
                base: self.base,
                id,
            })
    }

    /// Iterate over all cells that directly depend on this cell.
    pub fn each_dependent_cell(&self) -> impl Iterator<Item=CellRef<'a, H>> + '_ {
        self.base.each_dependent_cell(&self.id)
            .map(move |id| CellRef {
                base: self.base,
                id,
            })
    }
}


impl<'a, N: NetlistBase> CellRef<'a, N> {
    /// Iterate over the IDs of all pins of this cell.
    pub fn each_pin_id(&self) -> Box<dyn Iterator<Item=N::PinId> + '_> {
        self.base.each_pin(&self.id)
    }

    /// Iterate over all pins of this cell.
    pub fn each_pin(&self) -> impl Iterator<Item=PinRef<'_, N>> + '_ {
        self.base.each_pin(&self.id)
            .map(move |id| PinRef {
                base: self.base,
                id,
            })
    }

    /// Find a pin by it's name.
    pub fn pin_by_name(&self, name: &str) -> Option<PinRef<'_, N>> {
        self.base.pin_by_name(&self.id, name)
            .map(|id| {
                PinRef {
                    base: self.base,
                    id,
                }
            })
    }

    /// Iterate over all nets that live directly in this cell.
    pub fn each_net(&self) -> impl Iterator<Item=NetRef<'_, N>> + '_ {
        self.base.each_internal_net(&self.id)
            .map(move |id| NetRef {
                base: self.base,
                id
            })
    }
}


/// Default implementation for `CellInstRef`.
/// This is just a wrapper around a netlist and a cell ID.
pub struct CellInstRef<'a, H: HierarchyBase + ?Sized> {
    /// Reference to the parent netlist.
    base: &'a H,
    /// ID of the corresponding cell instance.
    id: H::CellInstId,
}

impl<'a, H: HierarchyBase> CellInstRef<'a, H> {
    /// Get the ID of this cell instance.
    pub fn id(&self) -> H::CellInstId {
        self.id.clone()
    }

    /// Get the name of the cell instance.
    pub fn name(&self) -> Option<H::NameType> {
        self.base.cell_instance_name(&self.id)
    }

    /// Get the parent cell of this instance.
    pub fn parent(&self) -> CellRef<'_, H> {
        CellRef {
            base: self.base,
            id: self.parent_id(),
        }
    }

    /// Get the template cell of this instance.
    pub fn template(&self) -> CellRef<'_, H> {
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

impl<'a, N: NetlistBase> CellInstRef<'a, N> {
    /// Iterate over the IDs of all pins of this cell.
    pub fn each_pin_instance_id(&self) -> Box<dyn Iterator<Item=N::PinInstId> + '_> {
        self.base.each_pin_instance(&self.id)
    }

    /// Iterate over all pins of this cell.
    pub fn each_pin_instance(&self) -> impl Iterator<Item=PinInstRef<'_, N>> + '_ {
        self.base.each_pin_instance(&self.id)
            .map(move |id| PinInstRef {
                base: self.base,
                id,
            })
    }

    /// Iterate over all nets are connected to this instance. A net might appear more than once.
    pub fn each_net(&self) -> impl Iterator<Item=NetRef<'_, N>> + '_ {
        self.base.each_external_net(&self.id)
            .map(move |id| NetRef {
                base: self.base,
                id
            })
    }
}

/// A reference to a net.
/// This is just a wrapper around a netlist and a net ID.
pub struct NetRef<'a, N: NetlistBase + ?Sized> {
    /// Reference to the parent data structure.
    base: &'a N,
    /// ID of the net.
    id: N::NetId,
}

impl<'a, N: NetlistBase> NetRef<'a, N> {
    /// Get the net ID.
    pub fn id(&self) -> N::NetId {
        self.id.clone()
    }

    /// Get the name of the net.
    pub fn name(&self) -> Option<N::NameType> {
        self.base.net_name(&self.id)
    }

    /// Get the cell where this net lives in.
    pub fn parent(&self) -> CellRef<'_, N> {
        CellRef {
            base: self.base,
            id: self.base.parent_cell_of_net(&self.id),
        }
    }

    /// Iterate over each pin attached to this net.
    pub fn each_pin(&self) -> impl Iterator<Item=PinRef<'_, N>> + '_ {
        self.base.each_pin_of_net(&self.id)
            .map(move |id| {
                PinRef {
                    base: self.base,
                    id,
                }
            })
    }

    /// Iterate over each pin instance attached to this net.
    pub fn each_pin_instance(&self) -> impl Iterator<Item=PinInstRef<'_, N>> + '_ {
        self.base.each_pin_instance_of_net(&self.id)
            .map(move |id| {
                PinInstRef {
                    base: self.base,
                    id,
                }
            })
    }

    /// Iterate over terminal attached to this net.
    pub fn each_terminal(&self) -> impl Iterator<Item=TerminalRef<'_, N>> + '_ {
        let pins = self.each_pin()
            .map(|p| p.into());
        let pin_insts = self.each_pin_instance()
            .map(|p| p.into());
        pins.chain(pin_insts)
    }
}

/// A reference to a pin.
/// This is just a wrapper around a netlist and a pin ID.
pub struct PinRef<'a, N: NetlistBase + ?Sized> {
    /// Reference to the parent data structure.
    base: &'a N,
    /// ID of the pin.
    id: N::PinId,
}

impl<'a, N: NetlistBase> PinRef<'a, N> {
    /// Get the pin ID.
    pub fn id(&self) -> N::PinId {
        self.id.clone()
    }

    /// Get the name of the pin.
    pub fn name(&self) -> N::NameType {
        self.base.pin_name(&self.id)
    }

    /// Get the net which is attached to the pin from inside the cell.
    pub fn net(&self) -> Option<NetRef<'_, N>> {
        self.base.net_of_pin(&self.id)
            .map(|id| NetRef {
                base: self.base,
                id,
            })
    }

    /// Get the cell which contains this pin.
    pub fn cell(&self) -> N::CellId {
        self.base.parent_cell_of_pin(&self.id)
    }
}

/// A reference to a pin instance.
/// This is just a wrapper around a netlist and a pin instance ID.
pub struct PinInstRef<'a, N: NetlistBase + ?Sized> {
    /// Reference to the parent data structure.
    base: &'a N,
    /// ID of the pin instance.
    id: N::PinInstId,
}

impl<'a, N: NetlistBase> PinInstRef<'a, N> {
    /// Get the pin instance ID.
    pub fn id(&self) -> N::PinInstId {
        self.id.clone()
    }

    /// Get the template of this pin instance.
    pub fn pin(&self) -> PinRef<'_, N> {
        PinRef {
            base: self.base,
            id: self.base.template_pin(&self.id),
        }
    }

    /// Get the parent cell instance.
    pub fn cell_instance(&self) -> CellInstRef<'a, N> {
        CellInstRef {
            base: self.base,
            id: self.base.parent_of_pin_instance(&self.id),
        }
    }

    /// Get the net which is attached to this pin instance.
    pub fn net(&self) -> Option<NetRef<'_, N>> {
        self.base.net_of_pin_instance(&self.id)
            .map(|id| NetRef {
                base: self.base,
                id,
            })
    }
}

/// Either a pin or a pin instance.
pub enum TerminalRef<'a, N: NetlistBase + ?Sized> {
    /// A template pin.
    Pin(PinRef<'a, N>),
    /// An instance of a pin.
    PinInst(PinInstRef<'a, N>),
}

impl<'a, N: NetlistBase> From<PinRef<'a, N>> for TerminalRef<'a, N> {
    fn from(p: PinRef<'a, N>) -> Self {
        Self::Pin(p)
    }
}

impl<'a, N: NetlistBase> From<PinInstRef<'a, N>> for TerminalRef<'a, N> {
    fn from(p: PinInstRef<'a, N>) -> Self {
        Self::PinInst(p)
    }
}


impl<'a, N: NetlistBase> TerminalRef<'a, N> {
    /// Get the attached net.
    pub fn net(&self) -> Option<NetRef<'_, N>> {
        match self {
            TerminalRef::Pin(p) => p.net(),
            TerminalRef::PinInst(p) => p.net()
        }
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