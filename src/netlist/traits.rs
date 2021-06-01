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

//! Traits for netlist data types.

use std::hash::Hash;
use crate::netlist::direction::Direction;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use itertools::Itertools;

/// A reference to a circuit.
pub trait CircuitRef {
    /// Netlist type.
    type N: NetlistBase;

    /// Get the ID of this circuit.
    fn id(&self) -> <<Self as CircuitRef>::N as NetlistBase>::CellId;

    /// Get the name of the circuit.
    fn name(&self) -> <<Self as CircuitRef>::N as NetlistBase>::NameType;

    /// Get the net of the logical constant zero.
    fn net_zero(&self) -> <<Self as CircuitRef>::N as NetlistBase>::NetId;

    /// Get the net of the logical constant one.
    fn net_one(&self) -> <<Self as CircuitRef>::N as NetlistBase>::NetId;
}

/// A reference to a circuit instance.
pub trait CircuitInstRef {
    /// Netlist type.
    type N: NetlistBase;

    /// Get the ID of this circuit.
    fn id(&self) -> <<Self as CircuitInstRef>::N as NetlistBase>::CellInstId;

    /// Get the name of the circuit.
    fn name(&self) -> Option<<<Self as CircuitInstRef>::N as NetlistBase>::NameType>;
}

/// A reference to a pin.
pub trait PinRef {
    /// Netlist type.
    type N: NetlistBase;
}

/// A reference to a pin instance.
pub trait PinInstRef {
    /// Netlist type.
    type N: NetlistBase;
}


/// A reference to a net.
pub trait NetRef {
    /// Netlist type.
    type N: NetlistBase;

    // /// Get the circuit where this net lives.
    // fn parent_circuit(self) -> CircuitRef<'a>;

    // /// Iterate over all external pin IDs connected to this net.
    // fn each_pin_id(&self) -> impl Iterator<Item=PinId> + '_;
    //
    // /// Iterate over all internal pin instance IDs connected to this net.
    // fn each_pin_inst_id(&self) -> impl Iterator<Item=PinInstId> + '_;
    //
    // /// Iterate over all external pins connected to this net.
    // fn each_pin_ref(&'a self) -> impl Iterator<Item=PinRef<'a>> + 'a;
    //
    // /// Iterate over all internal pins instances connected to this net.
    // fn each_pin_inst_ref(&'a self) -> Box<impl Iterator<Item=PinInstRef<'a>>> + 'a;
}

/// Default implementation for `CircuitRef`.
/// This is just a wrapper around a netlist and a circuit ID.
pub struct DefaultCircuitRef<'a, N: NetlistBase + ?Sized> {
    /// Reference to the parent netlist.
    netlist: &'a N,
    /// ID of the corresponding circuit.
    id: N::CellId,
}

impl<'a, N: NetlistBase> CircuitRef for DefaultCircuitRef<'a, N> {
    type N = N;

    fn id(&self) -> N::CellId {
        self.id.clone()
    }

    fn name(&self) -> N::NameType {
        self.netlist.circuit_name(&self.id)
    }

    fn net_zero(&self) -> N::NetId {
        self.netlist.net_zero(&self.id)
    }

    fn net_one(&self) -> N::NetId {
        self.netlist.net_one(&self.id)
    }
}


/// Default implementation for `CircuitInstRef`.
/// This is just a wrapper around a netlist and a circuit ID.
pub struct DefaultCircuitInstRef<'a, N: NetlistBase + ?Sized> {
    /// Reference to the parent netlist.
    netlist: &'a N,
    /// ID of the corresponding circuit instance.
    id: N::CellInstId,
}

impl<'a, N: NetlistBase> CircuitInstRef for DefaultCircuitInstRef<'a, N> {
    type N = N;

    fn id(&self) -> N::CellInstId {
        self.id.clone()
    }

    fn name(&self) -> Option<N::NameType> {
        self.netlist.circuit_instance_name(&self.id)
    }
}

/// Trait that provides object-like read access to a netlist and its elements.
pub trait NetlistReferenceAccess: NetlistBase
{
    /// Reference type for circuits.
    type CircuitRefType: CircuitRef;
    /// Reference type for circuit instances.
    type CircuitInstRefType: CircuitInstRef;
}

/// Most basic trait of a netlist.
///
/// ## Netlist component relations
///
/// A netlist consists of circuits which are templates for circuit instances.
/// Each circuit may contain such instances of other circuits.
///
/// The following diagram illustrates how this composition graph can be traversed using the functions
/// defined by `NetlistBase`.
///
/// ```txt
///                          each_circuit_dependency
///                      +---------------------------+
///                      |                           |
///                      +                           v
///       +----------------+ each_dependent_circuit +------------------+
///       |Circuit (Top)   |<----------------------+|Circuit (Sub)     |
///       +----------------+                        +------------------+
///       |+              ^|                        | ^   +            |
///       ||each_instance ||                        | |   |            |
///       ||              ||                        | |   |            |
///       ||              |parent_circuit           | |   |            |
///       ||              ||                        | |   |            |
///       ||+-----------+ ||                        | |   |            |
///  +--> |>|Inst1 (Sub)|-+|                        | |   |            |
///  |    ||+-----------+  |                        | |   |            |
///  |    ||               |                        | |   |            |
///  |    ||               |                        +-|---|------------+
///  |    ||               |                          |   |
///  |    ||+-----------+  |  template_circuit        |   |
///  +--> |>|Inst2 (Sub)|+----------------------------+   |
///  |    | +-----------+  |                              |
///  |    |                |                              |
///  |    |                |                              |
///  |    +----------------+                              |
///  |                                                    |
///  |                         each_reference             |
///  +----------------------------------------------------+
/// ```
///
pub trait NetlistBase {
    /// Type for names of circuits, instances, pins, etc.
    type NameType: Eq + Hash + From<String> + Into<String> + Clone
    + Borrow<String> + Borrow<str>
    + PartialOrd + Ord
    + std::fmt::Display + std::fmt::Debug;
    /// Pin identifier type.
    type PinId: Eq + Hash + Clone + std::fmt::Debug;
    /// Pin instance identifier type.
    /// A pin instance is a pin of a circuit instance.
    type PinInstId: Eq + Hash + Clone + std::fmt::Debug;
    /// Either a pin or a pin instance ID.
    type TerminalId: Eq + Hash + Clone + std::fmt::Debug;
    /// Circuit identifier type.
    type CellId: Eq + Hash + Clone + std::fmt::Debug;
    /// Circuit instance identifier type.
    type CellInstId: Eq + Hash + Clone + std::fmt::Debug;
    /// Net identifier type.
    type NetId: Eq + Hash + Clone + std::fmt::Debug;


    /// Create a new empty netlist.
    fn new() -> Self;

    /// Find a circuit by its name.
    /// Return the circuit with the given name. Returns `None` if the circuit does not exist.
    fn circuit_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N>;

    /// Find a circuit instance by its name.
    /// Returns `None` if the name does not exist.
    fn circuit_instance_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::CellInstId>
        where Self::NameType: Borrow<N>;

    /// Get the ID of the template circuit of this instance.
    fn template_circuit(&self, circuit_instance: &Self::CellInstId) -> Self::CellId;

    /// Get the ID of the template pin of this pin instance.
    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId;

    /// Get the signal direction of the pin.
    fn pin_direction(&self, pin: &Self::PinId) -> Direction;

    /// Get the name of the pin.
    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType;

    /// Find a pin by its name.
    /// Returns `None` if no such pin can be found.
    fn pin_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::PinId>
        where Self::NameType: Borrow<N>;

    /// Get the ID of the parent circuit of this instance.
    fn parent_circuit(&self, circuit_instance: &Self::CellInstId) -> Self::CellId;

    /// Get the ID of the parent circuit of this pin.
    fn parent_circuit_of_pin(&self, pin: &Self::PinId) -> Self::CellId;

    /// Get the ID of the circuit instance that holds this pin instance.
    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CellInstId;

    /// Get the internal net attached to this pin.
    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId>;

    /// Get the external net attached to this pin instance.
    fn net_of_pin_instance(&self, pin_instance: &Self::PinInstId) -> Option<Self::NetId>;

    /// Get the net of the logical constant zero.
    fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId;

    /// Get the net of the logical constant one.
    fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId;

    /// Find a net by its name inside the parent circuit.
    /// Returns `None` if no such net can be found.
    fn net_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::NetId>
        where Self::NameType: Borrow<N>;

    /// Get the name of the net.
    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType>;

    /// Get the name of the circuit.
    fn circuit_name(&self, circuit: &Self::CellId) -> Self::NameType;

    /// Get the name of the circuit instance.
    fn circuit_instance_name(&self, circuit_inst: &Self::CellInstId) -> Option<Self::NameType>;

    /// Call a function on each circuit of the netlist.
    fn for_each_circuit<F>(&self, f: F) where F: FnMut(Self::CellId) -> ();

    /// Get a `Vec` of all circuit IDs in this netlist.
    fn each_circuit_vec(&self) -> Vec<Self::CellId> {
        let mut v = Vec::new();
        self.for_each_circuit(|c| v.push(c.clone()));
        v
    }

    /// Iterate over all circuits.
    fn each_circuit<'a>(&'a self) -> Box<dyn Iterator<Item=Self::CellId> + 'a> {
        Box::new(self.each_circuit_vec().into_iter())
    }

    /// Call a function on each instance in this circuit.
    fn for_each_instance<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> ();

    /// Get a `Vec` of the IDs of all instances in this circuit.
    fn each_instance_vec(&self, circuit: &Self::CellId) -> Vec<Self::CellInstId> {
        let mut v = Vec::new();
        self.for_each_instance(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all instances in a circuit.
    fn each_instance<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + 'a> {
        Box::new(self.each_instance_vec(circuit).into_iter())
    }

    /// Call a function for each circuit that is a child of this `circuit`.
    fn for_each_circuit_dependency<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> ();

    /// Get a `Vec` of each circuit that is a child of this `circuit`.
    fn each_circuit_dependency_vec(&self, circuit: &Self::CellId) -> Vec<Self::CellId> {
        let mut v = Vec::new();
        self.for_each_circuit_dependency(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all circuits that are childs of this `circuit`.
    fn each_circuit_dependency<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a> {
        Box::new(self.each_circuit_dependency_vec(circuit).into_iter())
    }

    /// Call a function for each circuit that directly depends on `circuit`.
    fn for_each_dependent_circuit<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> ();

    /// Get a `Vec` of each circuit that directly depends on `circuit`.
    fn each_dependent_circuit_vec(&self, circuit: &Self::CellId) -> Vec<Self::CellId> {
        let mut v = Vec::new();
        self.for_each_dependent_circuit(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over each circuit that directly depends on `circuit`.
    fn each_dependent_circuit<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + 'a> {
        Box::new(self.each_dependent_circuit_vec(circuit).into_iter())
    }

    /// Iterate over all instances of this `circuit`, i.e. instances that use this circuit as
    /// a template.
    fn for_each_reference<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> ();

    /// Get a `Vec` with all circuit instances referencing this circuit.
    fn each_reference_vec(&self, circuit: &Self::CellId) -> Vec<Self::CellInstId> {
        let mut v = Vec::new();
        self.for_each_reference(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all instances of this `circuit`, i.e. instances that use this circuit as
    /// a template.
    fn each_reference<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + 'a> {
        Box::new(self.each_reference_vec(circuit).into_iter())
    }

    /// Call a function for each pin of the circuit.
    fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> ();

    /// Get a `Vec` with the IDs of all pins of this circuit.
    fn each_pin_vec(&self, circuit: &Self::CellId) -> Vec<Self::PinId> {
        let mut v = Vec::new();
        self.for_each_pin(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a circuit.
    fn each_pin<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.each_pin_vec(circuit).into_iter())
    }


    /// Call a function for each pin instance of the circuit instance.
    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CellInstId, f: F) where F: FnMut(Self::PinInstId) -> ();

    /// Get a `Vec` with the IDs of all pin instance of this circuit instance.
    fn each_pin_instance_vec(&self, circuit_instance: &Self::CellInstId) -> Vec<Self::PinInstId> {
        let mut v = Vec::new();
        self.for_each_pin_instance(circuit_instance, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pin instances of a circuit.
    fn each_pin_instance<'a>(&'a self, circuit_instance: &Self::CellInstId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.each_pin_instance_vec(circuit_instance).into_iter())
    }


    /// Iterate over all external nets connected to the circuit instance.
    /// A net might appear more than once.
    fn each_external_net<'a>(&'a self, circuit_instance: &Self::CellInstId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_pin_instance(circuit_instance)
            .flat_map(move |pin_id| self.net_of_pin_instance(&pin_id)))
    }

    /// Call a function for net of the circuit.
    fn for_each_internal_net<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::NetId) -> ();

    /// Get a `Vec` with all nets in this circuit.
    fn each_internal_net_vec(&self, circuit: &Self::CellId) -> Vec<Self::NetId> {
        let mut v = Vec::new();
        self.for_each_internal_net(circuit, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all defined nets inside a circuit.
    fn each_internal_net<'a>(&'a self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::NetId> + 'a> {
        Box::new(self.each_internal_net_vec(circuit).into_iter())
    }

    /// Get the number of pins that are connected to this net.
    fn num_net_pins(&self, net: &Self::NetId) -> usize {
        let mut n = 0;
        self.for_each_pin_of_net(net, |_| n += 1);
        n
    }

    /// Get the number of pin instances that are connected to this net.
    fn num_net_pin_instances(&self, net: &Self::NetId) -> usize {
        let mut n = 0;
        self.for_each_pin_instance_of_net(net, |_| n += 1);
        n
    }

    /// Get the number of terminals that are connected to this net.
    fn num_net_terminals(&self, net: &Self::NetId) -> usize {
        self.num_net_pins(net) + self.num_net_pin_instances(net)
    }

    /// Get the number of cell instances inside the `circuit`.
    fn num_child_instances(&self, circuit: &Self::CellId) -> usize;

    /// Get the number of cells inside in this netlist.
    fn num_circuits(&self) -> usize;

    /// Get the number of pins of a circuit.
    fn num_pins(&self, circuit: &Self::CellId) -> usize;

    /// Get the number of references that point to this circuit, i.e. the number of
    /// instances of this circuit.
    fn num_references(&self, circuit: &Self::CellId) -> usize {
        self.each_reference(circuit).count()
    }

    /// Call a function for each pin connected to this net.
    fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> ();

    /// Get a `Vec` with all pin IDs connected to this net.
    fn each_pin_of_net_vec(&self, net: &Self::NetId) -> Vec<Self::PinId> {
        let mut v = Vec::new();
        self.for_each_pin_of_net(net, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a net.
    fn each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.each_pin_of_net_vec(net).into_iter())
    }


    /// Call a function for each pin instance connected to this net.
    fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> ();

    /// Get a `Vec` with all pin instance IDs connected to this net.
    fn each_pin_instance_of_net_vec(&self, net: &Self::NetId) -> Vec<Self::PinInstId> {
        let mut v = Vec::new();
        self.for_each_pin_instance_of_net(net, |c| v.push(c.clone()));
        v
    }

    /// Iterate over all pins of a net.
    fn each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.each_pin_instance_of_net_vec(net).into_iter())
    }

    /// Visit all circuit instances connected to this net.
    /// An instance is touched not more than once.
    fn for_each_circuit_instance_of_net<F>(&self, net: &Self::NetId, mut f: F) where F: FnMut(Self::CellInstId) -> () {
        let mut visited = HashSet::new();
        self.for_each_pin_instance_of_net(net, |pin_inst| {
            let inst = self.parent_of_pin_instance(&pin_inst);
            if !visited.contains(&inst) {
                f(inst);
            } else {
                visited.insert(inst);
            }
        })
    }

    /// Iterate over all circuit instances connected to this net.
    /// An instance is touched not more than once.
    fn each_circuit_instance_of_net_vec(&self, net: &Self::NetId) -> Vec<Self::CellInstId> {
        let mut v = Vec::new();
        self.for_each_circuit_instance_of_net(net, |c| v.push(c.clone()));
        v
    }


    /// Return a reference to the circuit with this ID.
    fn circuit(&self, id: Self::CellId) -> Box<dyn CircuitRef<N=Self> + '_>
        where Self: Sized {
        // TODO: Check that ID exists.
        Box::new(DefaultCircuitRef {
            netlist: self,
            id,
        })
    }

    /// Return a reference to the circuit instance with this ID.
    fn circuit_inst(&self, id: Self::CellInstId) -> Box<dyn CircuitInstRef<N=Self> + '_>
        where Self: Sized {
        // TODO: Check that ID exists.
        Box::new(DefaultCircuitInstRef {
            netlist: self,
            id,
        })
    }

    /// Write the netlist in a human readable form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let circuits = self.each_circuit_vec();
        // circuits.sort_by_key(|c| c.id());
        for c in &circuits {
            let circuit_name = self.circuit_name(c);
            let circuit_instances = self.each_instance_vec(c);
            // circuit_instances.sort_by_key(|c| c.id());

            // Get pin names together with the net they are connected to.
            let pin_names = self.each_pin(c)
                .map(|pin| {
                    let pin_name = self.pin_name(&pin);
                    let net = self.net_of_pin(&pin);
                    let net_name: Option<String> = net.
                        map(|n| self.net_name(&n)
                            .map(|n| n.into())
                            .unwrap_or("<unnamed>".into())); // TODO: Create a name.
                    format!("{}={:?}", pin_name, net_name)
                }).join(" ");

            writeln!(f, ".subckt {} {}", circuit_name, pin_names)?;
            for inst in &circuit_instances {
                // fmt::Debug::fmt(Rc::deref(&c), f)?;
                let sub_name: String = self.circuit_instance_name(inst)
                    .map(|n| n.into())
                    .unwrap_or("<unnamed>".into()); // TODO: Create a name.
                let sub_template = self.template_circuit(inst);
                let template_name = self.circuit_name(&sub_template);
                let nets = self.each_pin_instance(inst)
                    .map(|p| {
                        let pin = self.template_pin(&p);
                        let pin_name = self.pin_name(&pin);
                        let net = self.net_of_pin_instance(&p);
                        let net_name: Option<String> = net.
                            map(|n| self.net_name(&n)
                                .map(|n| n.into())
                                .unwrap_or("<unnamed>".into()));
                        format!("{}={:?}", pin_name, net_name)
                    }).join(" ");
                writeln!(f, "    X{} {} {}", sub_name, template_name, nets)?;
            }
            writeln!(f, ".ends {}\n", circuit_name)?;
        }
        fmt::Result::Ok(())
    }
}


/// Trait for netlists that support editing.
pub trait NetlistEdit: NetlistBase {
    /// Create a new and empty circuit.
    fn create_circuit(&mut self, name: Self::NameType, pins: Vec<(Self::NameType, Direction)>) -> Self::CellId;

    /// Delete the given circuit if it exists.
    fn remove_circuit(&mut self, circuit_id: &Self::CellId);


    /// Create a new circuit instance.
    fn create_circuit_instance(&mut self,
                               parent_circuit: &Self::CellId,
                               template_circuit: &Self::CellId,
                               name: Option<Self::NameType>) -> Self::CellInstId;

    /// Remove circuit instance if it exists.
    fn remove_circuit_instance(&mut self, id: &Self::CellInstId);

    /// Create a net net that lives in the `parent` circuit.
    fn create_net(&mut self, parent: &Self::CellId,
                  name: Option<Self::NameType>) -> Self::NetId;

    /// Set a new name for the net. This might panic if the name already exists.
    fn rename_net(&mut self, parent_circuit: &Self::CellId,
                  net_id: &Self::NetId,
                  new_name: Option<Self::NameType>);

    /// Delete the net if it exists and disconnect all connected terminals.
    fn remove_net(&mut self, net: &Self::NetId);

    /// Connect a pin to a net.
    /// Returns the old connected net, if any.
    fn connect_pin(&mut self, pin: &Self::PinId, net: Option<Self::NetId>) -> Option<Self::NetId>;

    /// Disconnect the pin from any connected net.
    /// Returns the old connected net, if any.
    fn disconnect_pin(&mut self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.connect_pin(pin, None)
    }

    /// Connect a pin instance to a net.
    /// Returns the old connected net, if any.
    fn connect_pin_instance(&mut self, pin: &Self::PinInstId, net: Option<Self::NetId>) -> Option<Self::NetId>;

    /// Disconnect the pin instance from any connected net.
    /// Returns the old connected net, if any.
    fn disconnect_pin_instance(&mut self, pin_instance: &Self::PinInstId) -> Option<Self::NetId> {
        self.connect_pin_instance(pin_instance, None)
    }

    /// Take all terminals that are connected to `old_net` and connect them to `new_net` instead.
    /// The old net is no longer used and removed.
    ///
    /// This is a default implementation that can possibly be implemented more efficiently for a concrete
    /// netlist type.
    fn replace_net(&mut self, old_net: &Self::NetId, new_net: &Self::NetId) {
        // Check that the nets live in this circuit.
        // TODO:
        // assert!(old_net.parent_circuit().ptr_eq(&self.self_reference()));
        // assert!(new_net.parent_circuit().ptr_eq(&self.self_reference()));
        // assert!(self.nets.borrow().contains_key(&old_net.id), "Old net does not exist in this circuit.");
        // assert!(self.nets.borrow().contains_key(&new_net.id), "New net does not exist in this circuit.");

        // Get terminals connected to the old net.
        let terminals: Vec<_> = self.each_pin_of_net(&old_net).collect();
        // Connect each terminal to the new net.
        for pin in terminals {
            self.connect_pin(&pin, Some(new_net.clone()));
        }
        // Get terminals connected to the old net.
        let terminals: Vec<_> = self.each_pin_instance_of_net(&old_net).collect();
        // Connect each terminal to the new net.
        for pin in terminals {
            self.connect_pin_instance(&pin, Some(new_net.clone()));
        }

        // Remove the now unused old net.
        self.remove_net(&old_net);
    }

    /// Replace the circuit instance with its contents. Remove the circuit instance afterwards.
    /// Does not purge nets nor unconnected instances.
    /// So there could be unconnected nets or unconnected instances.
    ///
    /// Nets keep their names if possible. If the net name already exists in this circuit, the name will
    /// be set to `None`.
    ///
    /// The content of the circuit instance will be renamed by appending the names like a path.
    fn flatten_circuit_instance(&mut self, circuit_instance: &Self::CellInstId) {
        // assert!(self.contains_instance(circuit_instance),
        //         "Instance does not live in this circuit.");

        // Get the template circuit.
        let template = self.template_circuit(circuit_instance);
        let parent_circuit = self.parent_circuit(circuit_instance);

        assert!(template != parent_circuit);

        // Mapping from old to new nets.
        let mut net_mapping: HashMap<Self::NetId, Self::NetId> = HashMap::new();

        // Get or create a new net as an equivalent of the old.
        let mut get_new_net = |netlist: &mut Self, old_net: &Self::NetId| -> Self::NetId {
            if let Some(net_net) = net_mapping.get(old_net) {
                net_net.clone()
            } else {
                // Get the name of the net.
                let net_name = netlist.net_name(old_net);
                // Resolve net name collisions.
                // It is possible that the net name already exists in this circuit.
                let net_name = if let Some(net_name) = net_name {
                    // Check if net name already exists.
                    if let Some(_) = netlist.net_by_name(&parent_circuit, &net_name) {
                        // Net name already exists in this circuit.
                        // Don't use it.
                        // TODO: Create a qualified name?
                        None
                    } else {
                        // Net name does not yet exist.
                        // We can use the original one.
                        Some(net_name)
                    }
                } else {
                    // No net name was given.
                    None
                };

                // Create the new net.
                let new_net = netlist.create_net(&parent_circuit, net_name);
                // Remember the mapping old_net -> net_net.
                net_mapping.insert(old_net.clone(), new_net.clone());
                new_net
            }
        };

        // Copy all sub instances into this circuit.
        // And connect their pins to copies of the original nets.
        let all_instances = self.each_instance_vec(&template);
        for sub in all_instances {
            let sub_template = self.template_circuit(&sub);

            let new_name = if let (Some(sub_instance_name), Some(inst_name)) =
            (self.circuit_instance_name(&sub), self.circuit_instance_name(circuit_instance)) {
                // Construct name for the new sub instance.
                // Something like: INSTANCE_TO_BE_FLATTENED:SUB_INSTANCE{_COUNTER}
                {
                    let mut new_name = format!("{}:{}", inst_name, sub_instance_name);
                    let mut i = 0;
                    // It is possible that the instance name already exists in this circuit.
                    // If this name too already exists, append a counter.
                    while self.circuit_instance_by_name(&parent_circuit, &new_name).is_some() {
                        new_name = format!("{}:{}_{}", inst_name, sub_instance_name, i);
                        i += 1;
                    }
                    Some(new_name)
                }
            } else {
                None
            };
            let new_inst = self.create_circuit_instance(&parent_circuit, &sub_template,
                                                        new_name.map(|n| n.into()));

            // Re-connect pins to copies of the original nets.
            // Loop over old/new pin instances.
            let pin_mapping: Vec<_> = self.each_pin_instance(&sub)
                .zip(self.each_pin_instance(&new_inst))
                .collect();
            for (old_pin, new_pin) in pin_mapping {
                // Get net on old pin.
                if let Some(old_net) = self.net_of_pin_instance(&old_pin) {
                    let new_net = get_new_net(self, &old_net);
                    self.connect_pin_instance(&new_pin, Some(new_net));
                }
            }
        }

        // Connect the newly created sub-instances and nets to this circuit
        // according to the old connections to the instance which is about to be flattened.
        {
            // First create a the mapping from inner nets to outer nets.
            // This is necessary for the case when multiple internal pins are connected to the same
            // internal net.
            let net_replacement_mapping: HashMap<_, _> = self.each_pin_instance_vec(circuit_instance)
                .into_iter()
                .filter_map(|old_pin| {
                    let outer_old_net = self.net_of_pin_instance(&old_pin);
                    let inner_old_net = self.net_of_pin(&self.template_pin(&old_pin));
                    // If the pin was either not connected on the outside or not connected inside, nothing
                    // needs to be done.
                    if let (Some(outer_net), Some(inner_old_net)) = (outer_old_net, inner_old_net) {
                        // Get the new copy of the inner net.
                        let inner_new_net = get_new_net(self, &inner_old_net);
                        // Attach the new inner net to the outer net (by replacement).
                        Some((inner_new_net, outer_net))
                    } else {
                        // Inner and outer pin were not connected, nothing needs to be done.
                        None
                    }
                })
                .collect();
            // Make the net replacement.
            net_replacement_mapping.iter()
                .for_each(|(inner_new_net, outer_net)|
                    self.replace_net(inner_new_net, outer_net)
                );
        }


        // Remove old instance.
        self.remove_circuit_instance(circuit_instance);

        // TODO: Clean-up.
        // self.purge_nets();
        // Remove unconnected instances.
    }

    /// Flatten all instances of this circuit by replacing them with their content.
    /// Remove the circuit from the netlist afterwards.
    /// For top level circuits this is equivalent to removing them.
    fn flatten_circuit(&mut self, circuit: &Self::CellId) {
        // TODO: Assert that the circuit lives in this netlist.
        // Get all instances of the circuit.

        // Flatten all instances of the circuit.
        for r in self.each_reference_vec(circuit) {
            self.flatten_circuit_instance(&r);
        }

        debug_assert_eq!(self.each_reference(circuit).count(), 0,
                         "Circuit should not have any references anymore.");

        // Remove the circuit.
        self.remove_circuit(circuit);
    }

    /// Delete all unconnected nets in this circuit.
    /// Return number of purged nets.
    fn purge_nets_in_circuit(&mut self, circuit_id: &Self::CellId) -> usize {
        let mut unused = Vec::new();
        self.for_each_internal_net(circuit_id, |n| {
            if self.num_net_terminals(&n) == 0 {
                unused.push(n)
            }
        });

        unused.iter()
            .for_each(|n| self.remove_net(n));

        return unused.len();
    }

    /// Delete all unconnected nets in all circuits.
    /// Return number of purged nets.
    fn purge_nets(&mut self) -> usize {
        let all_circuits = self.each_circuit_vec();
        all_circuits.iter()
            .map(|c| self.purge_nets_in_circuit(c))
            .sum()
    }
}