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
//! Wrappers around the `HierarchyBase` and `NetlistBase` traits which
//! provide more object like access methods.
//!

use crate::traits::{HierarchyBase, NetlistBase, LayoutBase};
use crate::prelude::{TerminalId, SimpleTransform};
use crate::netlist::direction::Direction;

/// Trait that provides object-like read access to a cell hierarchy structure and its elements.
pub trait HierarchyReferenceAccess: HierarchyBase
{
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

/// Trait that provides object-like read access to a hierarchical netlist structure and its elements.
pub trait NetlistReferenceAccess: NetlistBase {
    /// Get a reference to a pin from a pin ID.
    fn pin_ref(&self, pin: &Self::PinId) -> PinRef<'_, Self> {
        PinRef {
            base: self,
            id: pin.clone(),
        }
    }

    /// Get a reference to a pin instance.
    fn pin_instance_ref(&self, id: &Self::PinInstId) -> PinInstRef<'_, Self> {
        PinInstRef {
            base: self,
            id: id.clone(),
        }
    }

    /// Get a reference to a net.
    fn net_ref(&self, net: &Self::NetId) -> NetRef<'_, Self> {
        NetRef {
            base: self,
            id: net.clone(),
        }
    }

    /// Get a reference to a terminal.
    fn terminal_ref(&self, t: &TerminalId<Self>) -> TerminalRef<Self> {
        match t {
            TerminalId::PinId(p) => TerminalRef::Pin(self.pin_ref(p)),
            TerminalId::PinInstId(p) => TerminalRef::PinInst(self.pin_instance_ref(p)),
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

    /// Get the number of cell instances inside the `cell`.
    pub fn num_child_instances(&self) -> usize {
        self.base.num_child_instances(&self.id)
    }
}


impl<'a, N: NetlistBase> CellRef<'a, N> {
    /// Iterate over the IDs of all pins of this cell.
    pub fn each_pin_id(&self) -> impl Iterator<Item=N::PinId> + '_ {
        self.base.each_pin(&self.id)
    }

    /// Iterate over all pins of this cell.
    pub fn each_pin(&self) -> impl Iterator<Item=PinRef<'a, N>> + '_ {
        self.base.each_pin(&self.id)
            .map(move |id| PinRef {
                base: self.base,
                id,
            })
    }

    /// Iterate over all input pins of this cell.
    pub fn each_input_pin(&self) -> impl Iterator<Item=PinRef<'a, N>> + '_ {
        self.each_pin().filter(|p| p.direction().is_input())
    }

    /// Iterate over all output pins of this cell.
    pub fn each_output_pin(&self) -> impl Iterator<Item=PinRef<'a, N>> + '_ {
        self.each_pin().filter(|p| p.direction().is_output())
    }

    /// Find a pin by it's name.
    pub fn pin_by_name(&self, name: &str) -> Option<PinRef<'a, N>> {
        self.base.pin_by_name(&self.id, name)
            .map(|id| {
                PinRef {
                    base: self.base,
                    id,
                }
            })
    }

    /// Iterate over all nets that live directly in this cell.
    pub fn each_net(&self) -> impl Iterator<Item=NetRef<'a, N>> + '_ {
        self.base.each_internal_net(&self.id)
            .map(move |id| NetRef {
                base: self.base,
                id,
            })
    }

    /// Find a net by its name.
    pub fn net_by_name(&self, name: &str) -> Option<NetRef<'a, N>> {
        self.base.net_by_name(&self.id, name)
            .map(|id| NetRef {
                base: self.base,
                id,
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

impl<'a, N: NetlistBase> CellInstRef<'a, N> {
    /// Iterate over the IDs of all pins of this cell.
    pub fn each_pin_instance_id(&self) -> impl Iterator<Item=N::PinInstId> + '_ {
        self.base.each_pin_instance(&self.id)
    }

    /// Iterate over all pins of this cell.
    pub fn each_pin_instance(&self) -> impl Iterator<Item=PinInstRef<'a, N>> + '_ {
        self.base.each_pin_instance(&self.id)
            .map(move |id| PinInstRef {
                base: self.base,
                id,
            })
    }

    /// Iterate over all nets are connected to this instance. A net might appear more than once.
    pub fn each_net(&self) -> impl Iterator<Item=NetRef<'a, N>> + '_ {
        self.base.each_external_net(&self.id)
            .map(move |id| NetRef {
                base: self.base,
                id,
            })
    }
}


impl<'a, L: LayoutBase> CellInstRef<'a, L> {
    /// Get the geometric transform that describes the location of a cell instance relative to its parent.
    pub fn get_transform(&self) -> SimpleTransform<L::Coord> {
        self.base().get_transform(&self.id)
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
    pub fn parent(&self) -> CellRef<'a, N> {
        CellRef {
            base: self.base,
            id: self.base.parent_cell_of_net(&self.id),
        }
    }

    /// Iterate over each pin attached to this net.
    pub fn each_pin(&self) -> impl Iterator<Item=PinRef<'a, N>> + '_ {
        self.base.each_pin_of_net(&self.id)
            .map(move |id| {
                PinRef {
                    base: self.base,
                    id,
                }
            })
    }

    /// Iterate over each pin instance attached to this net.
    pub fn each_pin_instance(&self) -> impl Iterator<Item=PinInstRef<'a, N>> + '_ {
        self.base.each_pin_instance_of_net(&self.id)
            .map(move |id| {
                PinInstRef {
                    base: self.base,
                    id,
                }
            })
    }

    /// Iterate over terminal attached to this net.
    pub fn each_terminal(&self) -> impl Iterator<Item=TerminalRef<'a, N>> + '_ {
        let pins = self.each_pin()
            .map(|p| p.into());
        let pin_insts = self.each_pin_instance()
            .map(|p| p.into());
        pins.chain(pin_insts)
    }

    /// Iterate over all terminals that drive the net. This should usually be one.
    /// Returns the pins that are marked as `inputs` and pin instances marked as `outputs`.
    /// Skips `InOut` terminals.
    pub fn each_driver(&self) -> impl Iterator<Item=TerminalRef<'a, N>> + '_ {
        self.each_terminal()
            .filter(|t| match t {
                TerminalRef::Pin(p) => p.direction().is_input(),
                TerminalRef::PinInst(p) => p.pin().direction().is_output()
            })
    }

    /// Iterate over all terminals that drive the net. This should usually be one.
    /// Returns the pins that are marked as `inputs` and pin instances marked as `outputs`.
    /// Skips `InOut` terminals.
    pub fn each_sink(&self) -> impl Iterator<Item=TerminalRef<'a, N>> + '_ {
        self.each_terminal()
            .filter(|t| match t {
                TerminalRef::Pin(p) => p.direction().is_output(),
                TerminalRef::PinInst(p) => p.pin().direction().is_input()
            })
    }

    /// Get a qualified name for this net.
    pub fn qname(&self, separator: &str) -> String {
        format!("{}{}{}", self.parent().name(), separator, self.name().unwrap_or_else(|| "<unnamed>".to_string().into()))
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

impl<'a, N: NetlistBase + ?Sized> Clone for PinRef<'a, N> {
    fn clone(&self) -> Self {
        Self {
            base: self.base,
            id: self.id.clone(),
        }
    }
}

impl<'a, N: NetlistBase> PinRef<'a, N> {
    /// Access the base structure.
    pub fn base(&self) -> &'_ N {
        self.base
    }

    /// Get the pin ID.
    pub fn id(&self) -> N::PinId {
        self.id.clone()
    }

    /// Get the terminal ID of this pin.
    pub fn terminal_id(&self) -> TerminalId<N> {
        TerminalId::PinId(self.id())
    }

    /// Get the name of the pin.
    pub fn name(&self) -> N::NameType {
        self.base.pin_name(&self.id)
    }

    /// Get the signal direction of the pin.
    pub fn direction(&self) -> Direction {
        self.base.pin_direction(&self.id)
    }

    /// Get the net which is attached to the pin from inside the cell.
    pub fn net(&self) -> Option<NetRef<'a, N>> {
        self.base.net_of_pin(&self.id)
            .map(|id| NetRef {
                base: self.base,
                id,
            })
    }

    /// Get the cell which contains this pin.
    pub fn cell(&self) -> CellRef<'a, N> {
        CellRef {
            base: self.base,
            id: self.base.parent_cell_of_pin(&self.id),
        }
    }

    /// Find the instance of this pin in the given cell instance.
    pub fn instance(&self, cell_inst: &N::CellInstId) -> PinInstRef<'a, N> {
        PinInstRef {
            base: self.base,
            id: self.base.pin_instance(cell_inst, &self.id),
        }
    }

    /// Convert the pin reference into a terminal reference.
    pub fn into_terminal(self) -> TerminalRef<'a, N> {
        self.into()
    }

    /// Create a qualified name.
    /// For pins: 'cell_name:pin_name'
    pub fn qname(&self, separator: &str) -> String {
        format!("{}{}{}", self.cell().name(), separator, self.name())
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

impl<'a, N: NetlistBase + ?Sized> Clone for PinInstRef<'a, N> {
    fn clone(&self) -> Self {
        Self {
            base: self.base,
            id: self.id.clone(),
        }
    }
}

impl<'a, N: NetlistBase> PinInstRef<'a, N> {
    /// Get the pin instance ID.
    pub fn id(&self) -> N::PinInstId {
        self.id.clone()
    }

    /// Get the terminal ID of this pin instance.
    pub fn terminal_id(&self) -> TerminalId<N> {
        TerminalId::PinInstId(self.id())
    }

    /// Get the template of this pin instance.
    pub fn pin(&self) -> PinRef<'a, N> {
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
    pub fn net(&self) -> Option<NetRef<'a, N>> {
        self.base.net_of_pin_instance(&self.id)
            .map(|id| NetRef {
                base: self.base,
                id,
            })
    }

    /// Convert the pin instance reference into a terminal reference.
    pub fn into_terminal(self) -> TerminalRef<'a, N> {
        self.into()
    }

    /// Create a qualified name.
    /// For pin instances: 'cell_name:cell_instance:pin_name'
    /// Where `:` is defined by `separator`.
    pub fn qname(&self, separator: &str) -> String {
        format!("{}{}{}{}{}",
                self.pin().cell().name(),
                separator,
                self.cell_instance().name().unwrap_or_else(|| "<unnamed>".to_string().into()),
                separator,
                self.pin().name())
    }
}

/// Either a pin or a pin instance.
pub enum TerminalRef<'a, N: NetlistBase + ?Sized> {
    /// A template pin.
    Pin(PinRef<'a, N>),
    /// An instance of a pin.
    PinInst(PinInstRef<'a, N>),
}

impl<'a, N: NetlistBase + ?Sized> Clone for TerminalRef<'a, N> {
    fn clone(&self) -> Self {
        match self {
            TerminalRef::Pin(p) => TerminalRef::Pin(p.clone()),
            TerminalRef::PinInst(p) => TerminalRef::PinInst(p.clone()),
        }
    }
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

impl<'a, N: NetlistBase> Into<TerminalId<N>> for TerminalRef<'a, N> {
    fn into(self) -> TerminalId<N> {
        match self {
            TerminalRef::Pin(p) => TerminalId::PinId(p.id),
            TerminalRef::PinInst(p) => TerminalId::PinInstId(p.id)
        }
    }
}

impl<'a, N: NetlistBase> TerminalRef<'a, N> {
    /// Get the ID of the terminal.
    pub fn id(&self) -> TerminalId<N> {
        (*self).clone().into()
    }

    /// Get the attached net.
    pub fn net(&self) -> Option<NetRef<'a, N>> {
        match self {
            TerminalRef::Pin(p) => p.net(),
            TerminalRef::PinInst(p) => p.net()
        }
    }

    /// Get the name of the pin.
    pub fn pin_name(&self) -> N::NameType {
        match self {
            TerminalRef::Pin(p) => p.name(),
            TerminalRef::PinInst(p) => p.pin().name()
        }
    }

    /// Get the parent cell of this terminal.
    /// For a pin, this equals the cell where the pin is defined.
    /// For a pin instance, this equals the parent of the cell instance which contains the pin instance.
    pub fn parent(&self) -> CellRef<N> {
        match self {
            TerminalRef::Pin(p) =>
                p.cell(),
            TerminalRef::PinInst(p) =>
                p.cell_instance().parent()
        }
    }

    /// Create a qualified name.
    /// For pins: 'cell_name:pin_name'
    /// For pin instances: 'cell_name:cell_instance:pin_name'
    /// Where `:` is defined by `separator`.
    pub fn qname(&self, separator: &str) -> String {
        match self {
            TerminalRef::Pin(p) =>
                p.qname(separator),
            TerminalRef::PinInst(p) =>
                p.qname(separator)
        }
    }
}

#[test]
fn test_chip_reference_access() {
    use crate::prelude::*;
    use crate::chip::Chip;

    let mut chip = Chip::new();
    let top = chip.create_cell("TOP".into());
    chip.create_pin(&top, "A".into(), Direction::Input);
    let sub = chip.create_cell("SUB".into());
    chip.create_pin(&sub, "B".into(), Direction::Input);
    let sub_inst1 = chip.create_cell_instance(&top, &sub, Some("inst1".into()));

    let top_ref = chip.cell_ref(&top);
    assert_eq!(&top_ref.id(), &top);

    let sub_inst1_ref = chip.cell_instance_ref(&sub_inst1);
    assert_eq!(&sub_inst1_ref.id(), &sub_inst1);
    assert_eq!(sub_inst1_ref.parent().id(), top_ref.id());
    assert_eq!(&sub_inst1_ref.template().id(), &sub);

    // Access nets and pins.
    assert_eq!(top_ref.each_net().count(), 2, "LOW and HIGH nets should be there.");
    assert_eq!(top_ref.each_pin().count(), 1);
    assert_eq!(sub_inst1_ref.each_pin_instance().count(), 1);
}