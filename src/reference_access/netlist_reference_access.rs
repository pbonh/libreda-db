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

use crate::traits::NetlistBase;
use crate::netlist::direction::Direction;
use crate::prelude::TerminalId;

use super::hierarchy_reference_access::*;

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


/// A reference to a net.
/// This is just a wrapper around a netlist and a net ID.
pub struct NetRef<'a, N: NetlistBase + ?Sized> {
    /// Reference to the parent data structure.
    pub(super) base: &'a N,
    /// ID of the net.
    pub(super) id: N::NetId,
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
    pub(super) base: &'a N,
    /// ID of the pin.
    pub(super) id: N::PinId,
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
    pub(super) base: &'a N,
    /// ID of the pin instance.
    pub(super) id: N::PinInstId,
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
