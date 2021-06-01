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
//! Chip data structure holding netlist and layout together.

// TODO: Remove this when fully implemented.
#![allow(unused_variables)]

use iron_shapes::CoordinateType;
use iron_shapes::shape::Geometry;
use iron_shapes::transform::SimpleTransform;

use crate::index::*;
use std::collections::HashMap;
use itertools::Itertools;
use std::borrow::Borrow;
use std::hash::Hash;
use crate::netlist::traits::{NetlistBase, NetlistEdit};
use crate::netlist::direction::Direction;
use crate::rc_string::RcString;
use std::fmt::Debug;
use std::ops::Deref;

use crate::property_storage::PropertyStore;
use crate::layout::traits::{LayoutEdit, LayoutBase};
use crate::layout::types::{LayerInfo};

// Use an alternative hasher that has better performance for integer keys.
use fnv::{FnvHashMap, FnvHashSet};
use crate::traits::HierarchyBase;

type IntHashMap<K, V> = FnvHashMap<K, V>;
type IntHashSet<V> = FnvHashSet<V>;

/// Default unsigned integer type.
pub type UInt = u32;
/// Default signed integer type.
pub type SInt = i32;

/// Integer coordinate type.
pub type Coord = i32;

/// Circuit identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CellId(u32);

/// Circuit instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CellInstId(usize);

/// Pin identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PinId(u32);

/// Pin instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PinInstId(usize);

/// Either a pin or pin instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TerminalId {
    /// Terminal is a pin.
    Pin(PinId),
    /// Terminal is a pin instance.
    PinInst(PinInstId),
}

impl From<PinId> for TerminalId {
    fn from(id: PinId) -> Self {
        TerminalId::Pin(id)
    }
}

impl From<PinInstId> for TerminalId {
    fn from(id: PinInstId) -> Self {
        TerminalId::PinInst(id)
    }
}

/// Net identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NetId(usize);
//
// /// Cell identifier.
// pub type CellId<T> = Index<Cell<T>>;
//
// /// Cell instance identifier.
// pub type CellInstId<T> = Index<CellInstance<T>>;
//

/// Unique (across layout) identifier of a shape.
pub type ShapeId = Index<Shape<Coord>, u32>;

/// ID for layers.
pub type LayerId = Index<LayerInfo, u16>;

/// A circuit is defined by an interface (pins) and
/// a content which consists of interconnected circuit instances.
///
/// Template parameters:
///
/// * `U`: User defined data.
#[derive(Debug, Clone)]
pub struct Circuit<C = Coord, U = ()>
    where C: CoordinateType, U: Default {
    /// ID of this circuit.
    id: CellId,
    /// Name of the circuit.
    name: RcString,

    /// Instances inside this circuit.
    instances: IntHashSet<CellInstId>,
    /// Instances inside this circuit indexed by name.
    /// Not every instance needs to have a name.
    instances_by_name: HashMap<RcString, CellInstId>,
    /// Circuit instances that reference to this circuit.
    references: IntHashSet<CellInstId>,
    /// All circuits that have instances of this circuit.
    parents: IntHashSet<CellId>,

    /// Set of circuits that are direct dependencies of this circuit.
    /// Stored together with a counter of how many instances of the dependency are present.
    /// This are the circuits towards the leaves in the dependency tree.
    dependencies: IntHashMap<CellId, usize>,
    /// Circuits that use a instance of this circuit.
    dependent_circuits: IntHashMap<CellId, usize>,

    /// Properties related to the instances in this template.
    /// Instance properties are stored here for lower overhead of cell instances.
    instance_properties: IntHashMap<CellInstId, PropertyStore<RcString>>,
    /// Properties related to this template.
    properties: PropertyStore<RcString>,
    /// User-defined data.
    user_data: U,

    // == Netlist == //

    /// Pin definitions, the actual pin structs are in the top level `Chip` struct.
    pins: Vec<PinId>,
    /// All nets in this circuit.
    nets: IntHashSet<NetId>,
    /// Nets IDs stored by name.
    nets_by_name: HashMap<RcString, NetId>,
    /// Logic constant LOW net.
    net_low: NetId,
    /// Logic constant HIGH net.
    net_high: NetId,

    // == Layout == //

    /// Mapping from layer indices to geometry data.
    shapes_map: IntHashMap<LayerId, Shapes<C>>,
}

impl Circuit {
    /// Get the ID of this circuit.
    pub fn id(&self) -> CellId {
        self.id
    }

    /// Get the name of this circuit.
    pub fn name(&self) -> &RcString {
        &self.name
    }

    /// Find a child instance in this circuit by its name.
    pub fn instance_id_by_name(&self, name: &str) -> Option<CellInstId> {
        self.instances_by_name.get(name).copied()
    }

    /// Iterate over the IDs of the child instances.
    pub fn each_instance_id(&self) -> impl Iterator<Item=CellInstId> + '_ + ExactSizeIterator {
        self.instances.iter().copied()
    }

    /// Iterate over the IDs of each dependency of this circuit.
    /// A dependency is a circuit that is instantiated in `self`.
    pub fn each_dependency_id(&self) -> impl Iterator<Item=CellId> + '_ + ExactSizeIterator {
        self.dependencies.keys().copied()
    }

    /// Iterate over the IDs of cell that depends on this circuit.
    pub fn each_dependent_cell_id(&self) -> impl Iterator<Item=CellId> + '_ + ExactSizeIterator {
        self.dependent_circuits.keys().copied()
    }

    /// Iterate over the IDs of all cells that hold instances of this circuit.
    pub fn each_parent(&self) -> impl Iterator<Item=CellId> + '_ + ExactSizeIterator {
        self.parents.iter().copied()
    }

    /// Iterate over the IDs of all instances of this circuit.
    pub fn each_reference(&self) -> impl Iterator<Item=CellInstId> + '_ + ExactSizeIterator {
        self.references.iter().copied()
    }

    // == Netlist == //

    /// Iterate over the IDs of the external circuit pins.
    pub fn each_pin_id(&self) -> impl Iterator<Item=PinId> + ExactSizeIterator + '_ {
        self.pins.iter().copied()
    }

    /// Return the number of pins of this circuit.
    pub fn num_pins(&self) -> usize {
        self.pins.len()
    }

    /// Get the ID of the pin at `position`.
    pub fn pin_id_at(&self, position: usize) -> PinId {
        self.pins[position]
    }

    /// Iterate over the IDs of all nets that are defined in this circuit.
    pub fn each_net_id(&self) -> impl Iterator<Item=NetId> + ExactSizeIterator + '_ {
        self.nets.iter().copied()
    }

    // == Layout == //

    /// Get the shape container of this layer.
    /// Returns `None` if the shapes object does not exist for this layer.
    pub fn shapes(&self, layer_id: &LayerId) -> Option<&Shapes<Coord>> {
        self.shapes_map.get(layer_id)
    }

    /// Get the mutable shape container of this layer.
    /// Returns `None` if the shapes object does not exist for this layer.
    fn shapes_mut(&mut self, layer_id: &LayerId) -> Option<&mut Shapes<Coord>> {
        self.shapes_map.get_mut(layer_id)
    }
}

/// Instance of a circuit.
///
/// Template parameters:
///
/// * `U`: User defined data.
#[derive(Debug, Clone)]
pub struct CircuitInst<C = Coord, U = ()>
    where C: CoordinateType {
    /// Name of the instance.
    name: Option<RcString>,
    /// The ID of the template circuit.
    template_circuit_id: CellId,
    /// The ID of the parent circuit where this instance lives in.
    parent_circuit_id: CellId,

    /// User-defined data.
    user_data: U,

    // == Netlist == //

    /// List of pins of this instance.
    pins: Vec<PinInstId>,

    // == Layout == //
    /// Transformation to put the cell to the right place an into the right scale/rotation.
    transform: SimpleTransform<C>,
    // TODO: Repetition
}

impl CircuitInst {
    /// Get the name of this instance.
    pub fn name(&self) -> &Option<RcString> {
        &self.name
    }

    /// Get ID of the pin instance at `position`.
    pub fn pin_inst_id_at(&self, position: usize) -> PinInstId {
        self.pins[position]
    }

    /// Get the ID of the template circuit.
    pub fn template_circuit_id(&self) -> CellId {
        self.template_circuit_id
    }

    /// Get the ID of the parent circuit.
    pub fn parent_circuit_id(&self) -> CellId {
        self.parent_circuit_id
    }

    // == Netlist == //

    /// Get a reference to the vector containing the pin instance IDs.
    pub fn pins(&self) -> &Vec<PinInstId> {
        &self.pins
    }

    /// Iterate over the IDs of the pin instances.
    pub fn each_pin_inst_id(&self) -> impl Iterator<Item=PinInstId> + ExactSizeIterator + '_ {
        self.pins.iter().copied()
    }

    // == Layout == //

    /// Get the transformation that represents the location and orientation of this instance.
    pub fn get_transform(&self) -> &SimpleTransform<Coord> {
        &self.transform
    }

    /// Set the transformation that represents the location and orientation of this instance.
    pub fn set_transform(&mut self, tf: SimpleTransform<Coord>) {
        self.transform = tf;
    }
}

/// Single bit wire pin.
#[derive(Debug, Clone)]
pub struct Pin {
    /// The unique ID of the pin.
    id: PinId,
    /// Name of the pin.
    name: RcString,
    /// Signal type/direction of the pin.
    direction: Direction,
    /// Parent circuit of this pin.
    circuit: CellId,
    /// Net that is connected to this pin.
    net: Option<NetId>,
    /// Position in the list of pins of the parent circuit.
    position: usize,

    // == Layout == //
    // /// List of shapes in the layout that represent the physical pin.
    // pin_shapes: Vec<(LayerId, ShapeId)>,
}

impl Pin {
    /// Get the name of the pin.
    pub fn name(&self) -> &RcString {
        &self.name
    }

    /// Get IO direction of this pin.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Get the circuit where this pin lives.
    pub fn parent_circuit(&self) -> CellId {
        self.circuit
    }

    /// Get the net that is internally connected to this pin.
    pub fn net(&self) -> Option<NetId> {
        self.net.clone()
    }

    /// Get the unique ID of this pin.
    pub fn id(&self) -> PinId {
        self.id
    }

    /// Get the position of this pin in the list of circuit pins.
    pub fn position(&self) -> usize {
        self.position
    }
}


/// Fat reference to a pin. Includes also a reference to the parent circuit.
pub struct PinRef<'a> {
    parent: CircuitRef<'a>,
    pin: &'a Pin,
}

impl<'a> PinRef<'a> {
    fn new(parent: CircuitRef<'a>, pin: &'a Pin) -> Self {
        Self {
            parent,
            pin,
        }
    }

    /// Get the circuit where this pin lives.
    pub fn parent_circuit(self) -> CircuitRef<'a> {
        self.parent
    }

    /// Get a reference to the netlist where this pin lives.
    pub fn netlist(self) -> &'a Chip {
        self.parent_circuit().netlist()
    }
}

impl Deref for PinRef<'_> {
    type Target = Pin;

    fn deref(&self) -> &Self::Target {
        self.pin
    }
}


/// Instance of a pin.
#[derive(Debug, Clone)]
pub struct PinInst {
    /// ID of the template pin.
    pub template_pin_id: PinId,
    /// Circuit instance where this pin instance lives in.
    pub circuit_inst: CellInstId,
    /// Net connected to this pin instance.
    net: Option<NetId>,
}

impl PinInst {
    /// Get the ID of the net that is internally connected to this pin.
    pub fn net_id(&self) -> Option<NetId> {
        self.net.clone()
    }
}

/// Fat reference to a pin instance. Includes also a reference to the parent circuit instance.
#[derive(Copy, Clone)]
pub struct PinInstRef<'a> {
    parent: CircuitInstanceRef<'a>,
    pin_inst: &'a PinInst,
}


impl<'a> PinInstRef<'a> {
    fn new(parent: CircuitInstanceRef<'a>, pin_inst: &'a PinInst) -> Self {
        Self {
            parent,
            pin_inst,
        }
    }

    /// Get the circuit instance where this pin lives.
    pub fn parent_circuit_instance(self) -> CircuitInstanceRef<'a> {
        self.parent
    }

    /// Get a reference to the template of this instance.
    pub fn template_pin(self) -> PinRef<'a> {
        PinRef::new(self.parent_circuit_instance().template(),
                    self.netlist().pin(&self.template_pin_id))
    }

    /// Get reference to the netlist where this instance lives.
    pub fn netlist(self) -> &'a Chip {
        self.parent_circuit_instance().netlist()
    }

    /// Get a reference to the net that is connected to this pin instance.
    pub fn net_ref(self) -> Option<NetRef<'a>> {
        self.net_id()
            .map(|id| self.netlist().net_ref(&id))
    }
}

impl Deref for PinInstRef<'_> {
    type Target = PinInst;

    fn deref(&self) -> &Self::Target {
        self.pin_inst
    }
}


/// A net represents an electric potential or a wire.
#[derive(Debug, Clone)]
pub struct Net {
    /// Name of the net.
    pub name: Option<RcString>,
    /// Parent circuit of the net.
    pub parent_id: CellId,
    /// Pins connected to this net.
    pub pins: IntHashSet<PinId>,
    /// Pin instances connected to this net.
    pub pin_instances: IntHashSet<PinInstId>,
}

impl Net {}

/// Fat reference to a net. Includes also a reference to the parent circuit.
#[derive(Copy, Clone, Debug)]
pub struct NetRef<'a> {
    parent: CircuitRef<'a>,
    net: &'a Net,
}

impl<'a> NetRef<'a> {
    fn new(parent: CircuitRef<'a>, net: &'a Net) -> Self {
        Self {
            parent,
            net,
        }
    }

    /// Get the circuit where this net lives.
    pub fn parent_circuit(self) -> CircuitRef<'a> {
        self.parent
    }

    /// Get reference to the netlist where this net lives.
    pub fn netlist(self) -> &'a Chip {
        self.parent_circuit().netlist()
    }

    /// Iterate over all external pin IDs connected to this net.
    pub fn each_pin_id(&self) -> impl Iterator<Item=PinId> + '_ {
        self.pins.iter().copied()
    }

    /// Iterate over all internal pin instance IDs connected to this net.
    pub fn each_pin_inst_id(&self) -> impl Iterator<Item=PinInstId> + '_ {
        self.pin_instances.iter().copied()
    }

    /// Iterate over all external pins connected to this net.
    pub fn each_pin_ref(&'a self) -> impl Iterator<Item=PinRef<'a>> + 'a {
        self.pins.iter()
            .map(move |id| {
                PinRef::new(self.parent_circuit(), self.netlist().pin(id))
            })
    }

    /// Iterate over all internal pins instances connected to this net.
    pub fn each_pin_inst_ref(&'a self) -> impl Iterator<Item=PinInstRef<'a>> + 'a {
        let netlist = self.netlist();
        self.pin_instances.iter()
            .map(move |id| {
                let pin_inst = netlist.pin_inst(id);
                let parent_inst_id = pin_inst.circuit_inst;
                let parent_inst = netlist.circuit_inst_ref(&parent_inst_id);
                PinInstRef::new(parent_inst, pin_inst)
            })
    }
}

impl Deref for NetRef<'_> {
    type Target = Net;

    fn deref(&self) -> &Self::Target {
        self.net
    }
}

/// A reference to a circuit instance.
///
/// This struct also keeps a reference to the parent netlist struct of the circuit.
#[derive(Copy, Clone, Debug)]
pub struct CircuitInstanceRef<'a> {
    netlist: &'a Chip,
    inst: &'a CircuitInst,
}

impl<'a> Deref for CircuitInstanceRef<'a> {
    type Target = CircuitInst;

    fn deref(&self) -> &Self::Target {
        self.inst
    }
}

impl<'a> CircuitInstanceRef<'a> {
    fn new(netlist: &'a Chip, inst: &'a CircuitInst) -> Self {
        Self {
            netlist,
            inst,
        }
    }
    /// Get reference to the netlist struct where this instance lives in.
    pub fn netlist(self) -> &'a Chip {
        self.netlist
    }

    /// Get a reference to the parent of this instance.
    pub fn parent(self) -> CircuitRef<'a> {
        let parent = &self.netlist.circuits[&self.parent_circuit_id];
        CircuitRef {
            netlist: self.netlist,
            circuit: parent,
        }
    }

    /// Get a reference to the template of this instance.
    pub fn template(self) -> CircuitRef<'a> {
        let template = &self.netlist.circuits[&self.template_circuit_id];
        CircuitRef {
            netlist: self.netlist,
            circuit: template,
        }
    }

    /// Get a reference to the pin instance at `position`.
    pub fn pin_inst_ref_at(self, position: usize) -> PinInstRef<'a> {
        let pin_inst_id = self.pins[position];
        PinInstRef::new(self, self.netlist().pin_inst(&pin_inst_id))
    }

    /// Iterate over each pin instance.
    pub fn each_pin_inst_ref(self) -> impl Iterator<Item=PinInstRef<'a>> + ExactSizeIterator + 'a {
        let num_pins = self.pins.len();
        (0..num_pins).map(move |pos| self.pin_inst_ref_at(pos))
    }

    /// Iterate over all nets that are connected externally to this circuit instance.
    /// Nets may appear multiple times.
    pub fn each_external_net(&self) -> impl Iterator<Item=NetRef<'a>> + 'a {
        self.each_pin_inst_ref()
            .filter_map(|p| p.net_ref())
    }
}


// /// Iterator wrapper.
// /// TODO
// pub struct PinRefIter<'a> {
//     circuit: CircuitRef<'a>
// }
//
// impl<'a> PinRefIter<'a> {
//     fn new(circuit: CircuitRef<'a>) -> Self {
//         Self { circuit }
//     }
// }
//
// impl<'a> IntoIterator for PinRefIter<'a> {
//     type Item = PinId;
//     type IntoIter = std::iter::Copied<std::slice::Iter<'a, PinId>>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.circuit.circuit.pins.iter().copied()
//     }
// }
//
// // impl<'a> Iterator for PinRefIter<'a> {
// //     type Item = PinId;
// //
// //     fn next(&mut self) -> Option<Self::Item> {
// //         unimplemented!()
// //     }
// // }

/// A netlist is the container of circuits.
#[derive(Debug, Default)]
pub struct Chip<C: CoordinateType = Coord> {
    circuits: IntHashMap<CellId, Circuit<C>>,
    circuits_by_name: HashMap<RcString, CellId>,
    circuit_instances: IntHashMap<CellInstId, CircuitInst>,
    nets: IntHashMap<NetId, Net>,
    pins: IntHashMap<PinId, Pin>,
    pin_instances: IntHashMap<PinInstId, PinInst>,

    id_counter_circuit: u32,
    id_counter_circuit_inst: usize,
    id_counter_pin: u32,
    id_counter_pin_inst: usize,
    id_counter_net: usize,

    // == Layout == //

    dbu: C,

    /// Counter for generating the next layer index.
    layer_index_generator: IndexGenerator<LayerInfo, u16>,
    /// Lookup table for finding layers by name.
    layers_by_name: HashMap<RcString, LayerId>,
    /// Lookup table for finding layers by index/datatype numbers.
    layers_by_index_datatype: IntHashMap<(UInt, UInt), LayerId>,
    /// Info structures for all layers.
    layer_info: IntHashMap<LayerId, LayerInfo>,
    /// ID generator for shapes.
    shape_index_generator: IndexGenerator<Shape<C>>,
}

impl Chip<Coord> {
    /// Find a circuit by its name.
    pub fn circuit_by_name<S: ?Sized + Eq + Hash>(&self, name: &S) -> Option<CellId>
        where RcString: Borrow<S> {
        self.circuits_by_name.get(name).copied()
    }

    /// Create a new circuit template.
    pub fn create_circuit(&mut self, name: RcString, pins: Vec<(RcString, Direction)>) -> CellId {
        assert!(!self.circuits_by_name.contains_key(&name), "Circuit with this name already exists.");
        let id = CellId(Self::next_id_counter_u32(&mut self.id_counter_circuit));

        // Create pins.
        let pins = pins.into_iter()
            .enumerate()
            .map(|(pos, (name, direction))|
                self.create_pin(id, name, direction, pos)
            )
            .collect();

        let circuit = Circuit {
            id,
            name: name.clone(),
            pins,
            instances: Default::default(),
            instances_by_name: Default::default(),
            references: Default::default(),
            parents: Default::default(),
            nets: Default::default(),
            nets_by_name: Default::default(),
            // Create LOW and HIGH nets.
            net_low: NetId(0),
            net_high: NetId(0),
            dependent_circuits: Default::default(),
            instance_properties: Default::default(),
            dependencies: Default::default(),
            user_data: Default::default(),
            shapes_map: Default::default(),
            properties: Default::default()
        };

        self.circuits.insert(id, circuit);
        self.circuits_by_name.insert(name, id);

        // Create LOW and HIGH nets.
        let net_low = self.create_net(&id, None);
        let net_high = self.create_net(&id, None);

        let c = self.circuit_mut(&id);
        c.net_low = net_low;
        c.net_high = net_high;

        id
    }

    /// Remove all instances inside the circuit, remove all instances of the circuit
    /// and remove finally the circuit itself.
    pub fn remove_circuit(&mut self, circuit_id: &CellId) {
        // Remove all instances inside this circuit.
        let instances = self.circuit(circuit_id).instances.iter().copied().collect_vec();
        for inst in instances {
            self.remove_circuit_instance(&inst);
        }
        // Remove all instances of this circuit.
        let references = self.circuit(circuit_id).references.iter().copied().collect_vec();
        for inst in references {
            self.remove_circuit_instance(&inst);
        }
        // Clean up pin definitions.
        let pins = self.circuit(circuit_id).pins.clone();
        for pin in pins {
            self.pins.remove(&pin).unwrap();
        }
        // Remove the circuit.
        let name = self.circuit(circuit_id).name.clone();
        self.circuits_by_name.remove(&name).unwrap();
        self.circuits.remove(&circuit_id).unwrap();
    }

    /// Create a new instance of `circuit_template` in the `parent` circuit.
    pub fn create_circuit_instance(&mut self, parent: &CellId,
                                   circuit_template: &CellId,
                                   name: Option<RcString>) -> CellInstId {
        let id = CellInstId(Self::next_id_counter_usize(&mut self.id_counter_circuit_inst));

        {
            // Check that creating this circuit instance does not create a cycle in the dependency graph.
            // There can be no recursive instances.
            let mut stack: Vec<CellId> = vec![*parent];
            while let Some(c) = stack.pop() {
                if &c == circuit_template {
                    // The circuit to be instantiated depends on the current circuit.
                    // This would insert a loop into the dependency tree.
                    // TODO: Don't panic but return an `Err`.
                    panic!("Cannot create recursive instances.");
                }
                // Follow the dependent circuits towards the root.
                stack.extend(self.circuit(&c).dependent_circuits.keys().copied())
            }
        }

        // Create pin instances from template pins.
        let pins = self.circuit(&circuit_template).pins.clone()
            .iter()
            .map(|&p| self.create_pin_inst(id, p))
            .collect();

        let inst = CircuitInst {
            name: name.clone(),
            template_circuit_id: *circuit_template,
            parent_circuit_id: *parent,
            user_data: (),
            pins: pins,
            transform: Default::default(),
        };

        self.circuit_instances.insert(id, inst);
        self.circuit_mut(&parent).instances.insert(id);
        self.circuit_mut(&circuit_template).references.insert(id);

        if let Some(name) = name {
            debug_assert!(!self.circuit(&parent).instances_by_name.contains_key(&name),
                          "Circuit instance name already exists.");
            self.circuit_mut(&parent).instances_by_name.insert(name, id);
        }

        // Remember dependency.
        {
            self.circuit_mut(&parent).dependencies.entry(*circuit_template)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        // Remember dependency.
        {
            self.circuit_mut(&circuit_template).dependent_circuits.entry(*parent)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        id
    }


    /// Remove a circuit instance after disconnecting it from the nets.
    pub fn remove_circuit_instance(&mut self, circuit_inst_id: &CellInstId) {
        // Disconnect all pins first.
        for pin in self.circuit_inst(circuit_inst_id).pins.clone() {
            self.disconnect_pin_instance(&pin);
        }
        // Remove the instance and all references.
        let parent = self.circuit_inst(&circuit_inst_id).parent_circuit_id;
        let template = self.circuit_inst(&circuit_inst_id).template_circuit_id;

        // Remove dependency.
        {
            // Decrement counter.
            let count = self.circuit_mut(&parent).dependencies.entry(template)
                .or_insert(0); // Should not happen.
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                self.circuit_mut(&parent).dependencies.remove(&template);
            }
        }

        // Remove dependency.
        {
            // Decrement counter.
            let count = self.circuit_mut(&template).dependent_circuits.entry(parent)
                .or_insert(0); // Should not happen.
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                self.circuit_mut(&template).dependent_circuits.remove(&parent);
            }
        }

        self.circuit_instances.remove(&circuit_inst_id).unwrap();
        self.circuit_mut(&parent).instances.remove(circuit_inst_id);
        self.circuit_mut(&template).references.remove(circuit_inst_id);
    }

    /// Create a new net in the `parent` circuit.
    pub fn create_net(&mut self, parent: &CellId, name: Option<RcString>) -> NetId {
        assert!(self.circuits.contains_key(parent));

        let id = NetId(Self::next_id_counter_usize(&mut self.id_counter_net));
        let net = Net {
            name: name.clone(),
            parent_id: *parent,
            pins: Default::default(),
            pin_instances: Default::default(),
        };
        self.nets.insert(id, net);
        let circuit = self.circuit_mut(parent);
        circuit.nets.insert(id);
        if let Some(name) = name {
            debug_assert!(!circuit.nets_by_name.contains_key(&name), "Net name already exists.");
            circuit.nets_by_name.insert(name, id);
        }
        id
    }

    pub fn rename_net(&mut self, parent_circuit: &CellId,
                      net_id: &NetId, new_name: Option<RcString>) {
        assert_eq!(parent_circuit, &self.nets.get(net_id).expect("Net not found.").parent_id);


        // Check if a net with this name already exists.
        if let Some(name) = &new_name {
            if let Some(other) = self.circuit(parent_circuit).nets_by_name.get(name) {
                if other != net_id {
                    panic!("Net name already exists.")
                } else {
                    return;
                }
            }
        }

        let maybe_old_name = self.net_mut(net_id).name.take();

        // Remove the old name mapping.
        if let Some(old_name) = maybe_old_name {
            self.circuit_mut(parent_circuit).nets_by_name.remove(&old_name);
        }

        // Add the new name mapping.
        if let Some(new_name) = new_name {
            self.nets.get_mut(net_id)
                .expect("Net not found.")
                .name.replace(new_name.clone());
            self.circuit_mut(parent_circuit).nets_by_name.insert(new_name, *net_id);
        }
    }

    /// Disconnect all connected terminals and remove the net.
    pub fn remove_net(&mut self, net: &NetId) {
        let pins = self.pins_for_net(net).collect_vec();
        let pin_insts = self.pins_instances_for_net(net).collect_vec();
        let parent_circuit = self.net(net).parent_id;

        for p in pins {
            self.disconnect_pin(&p);
        }
        for p in pin_insts {
            self.disconnect_pin_instance(&p);
        }
        let name = self.net(&net).name.clone();
        let circuit = self.circuit_mut(&parent_circuit);
        circuit.nets.remove(&net);
        if let Some(name) = &name {
            circuit.nets_by_name.remove(name).unwrap();
        }
        self.nets.remove(&net).unwrap();
    }

    /// Disconnect pin and return the ID of the net that was connected.
    pub fn disconnect_pin(&mut self, pin: &PinId) -> Option<NetId> {
        self.connect_pin(pin, None)
    }

    /// Connect the pin to a net.
    pub fn connect_pin(&mut self, pin: &PinId, net: Option<NetId>) -> Option<NetId> {
        if let Some(net) = net {
            // Sanity check.
            assert_eq!(self.pin(&pin).circuit, self.net(&net).parent_id,
                       "Pin and net do not live in the same circuit.");
        }

        let old_net = if let Some(net) = net {
            self.pin_mut(&pin).net.replace(net)
        } else {
            self.pin_mut(&pin).net.take()
        };

        if let Some(net) = old_net {
            // Remove the pin from the old net.
            self.net_mut(&net).pins.remove(&pin);
        }

        if let Some(net) = net {
            // Store the pin in the new net.
            self.net_mut(&net).pins.insert(*pin);
        }

        old_net
    }

    /// Disconnect the pin instance and return the net to which it was connected.
    pub fn disconnect_pin_instance(&mut self, pin: &PinInstId) -> Option<NetId> {
        self.connect_pin_instance(pin, None)
    }

    /// Connect the pin to a net.
    pub fn connect_pin_instance(&mut self, pin: &PinInstId, net: Option<NetId>) -> Option<NetId> {
        if let Some(net) = net {
            assert_eq!(self.circuit_inst(&self.pin_inst(pin).circuit_inst).parent_circuit_id,
                       self.net(&net).parent_id, "Pin and net do not live in the same circuit.");
        }

        let old_net = if let Some(net) = net {
            self.pin_inst_mut(&pin).net.replace(net)
        } else {
            self.pin_inst_mut(&pin).net.take()
        };

        if let Some(net) = old_net {
            // Remove the pin from the old net.
            self.net_mut(&net).pin_instances.remove(&pin);
        }

        if let Some(net) = net {
            // Store the pin in the new net.
            self.net_mut(&net).pin_instances.insert(*pin);
        }

        old_net
    }

    /// Get a circuit reference by its ID.
    pub fn circuit(&self, id: &CellId) -> &Circuit {
        &self.circuits[id]
    }

    /// Get a fat circuit reference by its ID.
    pub fn circuit_ref(&self, id: &CellId) -> CircuitRef {
        CircuitRef {
            netlist: self,
            circuit: &self.circuits[id],
        }
    }

    /// Get a mutable reference to the circuit by its ID.
    fn circuit_mut(&mut self, id: &CellId) -> &mut Circuit {
        self.circuits.get_mut(id).expect("Cell ID not found.")
    }

    /// Get a reference to a circuit instance.
    pub fn circuit_inst(&self, id: &CellInstId) -> &CircuitInst {
        &self.circuit_instances[id]
    }

    /// Get a mutable reference to a circuit instance.
    pub fn circuit_inst_mut(&mut self, id: &CellInstId) -> &mut CircuitInst {
        self.circuit_instances.get_mut(id).unwrap()
    }

    /// Get a fat circuit instance reference by its ID.
    pub fn circuit_inst_ref(&self, id: &CellInstId) -> CircuitInstanceRef {
        CircuitInstanceRef::new(self, &self.circuit_instances[id])
    }

    /// Get a reference to a net by its ID.
    pub fn net(&self, id: &NetId) -> &Net {
        &self.nets[id]
    }

    /// Get a mutable reference to a net by its ID.
    fn net_mut(&mut self, id: &NetId) -> &mut Net {
        self.nets.get_mut(id).unwrap()
    }

    /// Get a fat reference to the net by its ID.
    pub fn net_ref(&self, id: &NetId) -> NetRef {
        let net = self.net(id);
        let parent_circuit = self.circuit_ref(&net.parent_id);
        NetRef::new(parent_circuit, net)
    }

    /// Get a reference to a pin by its ID.
    pub fn pin(&self, id: &PinId) -> &Pin {
        &self.pins[id]
    }

    /// Get a mutable reference to a pin by its ID.
    fn pin_mut(&mut self, id: &PinId) -> &mut Pin {
        self.pins.get_mut(id).unwrap()
    }

    /// Get a reference to a pin instance by its ID.
    pub fn pin_inst(&self, id: &PinInstId) -> &PinInst {
        &self.pin_instances[id]
    }

    /// Get a mutable reference to a pin instance by its ID.
    fn pin_inst_mut(&mut self, id: &PinInstId) -> &mut PinInst {
        self.pin_instances.get_mut(id).unwrap()
    }

    /// Get the value of a counter and increment the counter afterwards.
    fn next_id_counter_usize(ctr: &mut usize) -> usize {
        let c = *ctr;
        *ctr += 1;
        c
    }

    /// Get the value of a counter and increment the counter afterwards.
    fn next_id_counter_u32(ctr: &mut u32) -> u32 {
        let c = *ctr;
        *ctr += 1;
        c
    }

    /// Append a new pin to the `parent` circuit.
    fn create_pin(&mut self, parent: CellId, name: RcString, direction: Direction, position: usize) -> PinId {
        let id = PinId(Self::next_id_counter_u32(&mut self.id_counter_pin));
        let pin = Pin {
            name,
            direction,
            circuit: parent,
            net: Default::default(),
            id,
            position,
        };
        self.pins.insert(id, pin);
        id
    }

    /// Insert a new pin instance to a circuit instance.
    fn create_pin_inst(&mut self, circuit: CellInstId, pin: PinId) -> PinInstId {
        let id = PinInstId(Self::next_id_counter_usize(&mut self.id_counter_pin_inst));
        let pin = PinInst {
            template_pin_id: pin,
            circuit_inst: circuit,
            net: None,
        };
        self.pin_instances.insert(id, pin);
        id
    }

    /// Get all nets that are connected to the circuit instance.
    pub fn circuit_inst_nets(&self, circuit_inst_id: &CellInstId) -> impl Iterator<Item=NetId> + '_ {
        self.circuit_inst(circuit_inst_id).pins.iter()
            .flat_map(move |p| self.pin_inst(p).net)
    }

    /// Iterate over all pins connected to a net.
    pub fn pins_for_net(&self, net: &NetId) -> impl Iterator<Item=PinId> + '_ {
        self.net(net).pins.iter().copied()
    }

    /// Iterate over all pin instances connected to a net.
    pub fn pins_instances_for_net(&self, net: &NetId) -> impl Iterator<Item=PinInstId> + '_ {
        self.net(net).pin_instances.iter().copied()
    }

    /// Iterate over all pins and pin instances connected to a net.
    pub fn terminals_for_net(&self, net: &NetId) -> impl Iterator<Item=TerminalId> + '_ {
        self.pins_for_net(net).map(|p| TerminalId::Pin(p))
            .chain(self.pins_instances_for_net(net).map(|p| TerminalId::PinInst(p)))
    }

    /// Remove all unconnected nets.
    /// Return number of purged nets.
    pub fn purge_nets(&mut self) -> usize {
        let unconnected: Vec<_> = self.nets.iter()
            .filter(|(_, n)| n.pin_instances.len() + n.pins.len() == 0)
            .map(|(&id, _)| id)
            .collect();
        let num = unconnected.len();
        for net in unconnected {
            self.remove_net(&net);
        }
        num
    }


    /// Return number of top level circuits (roots of the circuit tree).
    pub fn top_circuit_count(&self) -> usize {
        self.circuits.values()
            .filter(|c| c.parents.len() == 0)
            .count()
    }

    /// Iterate through all nets that are defined in the netlist.
    pub fn each_net(&self) -> impl Iterator<Item=NetId> + '_ {
        self.nets.keys().copied()
    }
}


/// A 'fat' reference to a circuit.
#[derive(Copy, Clone, Debug)]
pub struct CircuitRef<'a> {
    /// Reference to the parent netlist.
    netlist: &'a Chip,
    /// Reference to the circuit.
    circuit: &'a Circuit,
}

/// All functions of `Cell` are made available also for `CellRef` by implementation of the `Deref` trait.
impl<'a> Deref for CircuitRef<'a> {
    type Target = Circuit;

    fn deref(&self) -> &Self::Target {
        self.circuit
    }
}

impl<'a> CircuitRef<'a> {
    /// Iterate over all cell instances in this circuit.
    pub fn each_instance_ref(&self) -> impl Iterator<Item=CircuitInstanceRef<'_>> + ExactSizeIterator {
        self.instances.iter()
            .map(move |inst_id| {
                let inst = &self.netlist.circuit_instances[inst_id];
                CircuitInstanceRef {
                    netlist: self.netlist,
                    inst,
                }
            })
    }

    /// Find a child cell instance by its name.
    /// Returns `None` if no such instance exists.
    pub fn instance_ref_by_name(self, name: &str) -> Option<CircuitInstanceRef<'a>> {
        let id = self.instance_id_by_name(name);
        id.map(|id| {
            let inst = &self.netlist.circuit_instances[&id];
            CircuitInstanceRef {
                netlist: self.netlist,
                inst,
            }
        })
    }

    /// Iterate over the references to all cells that are dependencies of this cell.
    pub fn each_dependency_ref(&self) -> impl Iterator<Item=CircuitRef<'_>> + ExactSizeIterator {
        self.each_dependency_id()
            .map(move |id| CircuitRef {
                netlist: self.netlist,
                circuit: &self.netlist.circuits[&id],
            })
    }

    /// Iterate over the references to all cells that are dependent on this cell.
    pub fn each_dependent_cell_ref(&self) -> impl Iterator<Item=CircuitRef<'_>> + ExactSizeIterator {
        self.each_dependent_cell_id()
            .map(move |id| CircuitRef {
                netlist: self.netlist,
                circuit: &self.netlist.circuits[&id],
            })
    }

    /// Iterate over the IDs of the external circuit pins.
    pub fn each_pin_id(&'a self) -> impl Iterator<Item=PinId> + ExactSizeIterator + 'a {
        self.pins.iter().copied()
    }

    /// Iterate over each pin of this circuit.
    pub fn each_pin_ref(&self) -> impl Iterator<Item=PinRef<'_>> + ExactSizeIterator {
        self.each_pin_id()
            .map(move |id| PinRef::new(*self, &self.netlist.pin(&id)))
    }

    /// Get a reference to the pin at `position`.
    pub fn pin_ref_at(self, position: usize) -> PinRef<'a> {
        let pin_id = self.pin_id_at(position);
        PinRef::new(self, &self.netlist.pin(&pin_id))
    }

    /// Get a reference to the netlist where this circuit lives.
    pub fn netlist(self) -> &'a Chip {
        self.netlist
    }

    /// Iterate over all nets defined in this circuit.
    pub fn each_net_ref(&self) -> impl Iterator<Item=NetRef<'_>> {
        self.each_net_id()
            .map(move |id| NetRef::new(*self, self.netlist.net(&id)))
    }
}

#[test]
fn test_hashmap_netlist_reference_access() {
    let mut netlist = Chip::default();

    let a = netlist.create_circuit("A".into(), vec![
        ("A".into(), Direction::Input),
        ("Y".into(), Direction::Output)
    ]);
    let b = netlist.create_circuit("B".into(), vec![
        ("A".into(), Direction::Input),
        ("B".into(), Direction::Input),
        ("Y".into(), Direction::Output)
    ]);

    let inst_b1 = netlist.create_circuit_instance(&a, &b, None);
    let inst_b2 = netlist.create_circuit_instance(&a, &b, None);

    let a_ref = netlist.circuit_ref(&a);
    assert_eq!(a_ref.each_dependency_ref().count(), 1);
    assert_eq!(a_ref.each_dependent_cell_ref().count(), 0);

    let b_ref = netlist.circuit_ref(&b);
    assert_eq!(b_ref.each_dependency_ref().count(), 0);
    assert_eq!(b_ref.each_dependent_cell_ref().count(), 1);

    assert!(std::ptr::eq(&netlist, a_ref.netlist()));
    assert!(std::ptr::eq(&netlist, b_ref.netlist()));

    for inst in a_ref.each_instance_ref() {
        assert!(std::ptr::eq(&netlist, inst.netlist()));
        assert_eq!(a_ref.id(), inst.parent().id());

        assert_eq!(inst.template().num_pins(), inst.each_pin_inst_ref().count());
    }
}

impl NetlistBase for Chip {
    type NameType = RcString;
    type PinId = PinId;
    type PinInstId = PinInstId;
    type TerminalId = TerminalId;
    type CellId = CellId;
    type CellInstId = CellInstId;
    type NetId = NetId;

    /// Create an empty netlist.
    fn new() -> Self {
        Chip::default()
    }

    /// Find a circuit by its name.
    fn circuit_by_name<S: ?Sized + Eq + Hash>(&self, name: &S) -> Option<CellId>
        where Self::NameType: Borrow<S> {
        Chip::circuit_by_name(self, name)
    }

    fn circuit_instance_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N)
                                                       -> Option<Self::CellInstId>
        where Self::NameType: Borrow<N> {
        self.circuit(parent_circuit).instances_by_name.get(name).copied()
    }

    fn template_circuit(&self, circuit_instance: &Self::CellInstId) -> Self::CellId {
        self.circuit_inst(circuit_instance).template_circuit_id()
    }

    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId {
        self.pin_inst(pin_instance).template_pin_id
    }

    fn pin_direction(&self, pin: &Self::PinId) -> Direction {
        self.pin(pin).direction
    }

    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType {
        self.pin(pin).name.clone()
    }

    fn pin_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::PinId>
        where Self::NameType: Borrow<N> {
        // TODO: Create index for pin names.
        self.circuit(&parent_circuit).pins.iter().find(|p| self.pin(*p).name.borrow() == name)
            .copied()
    }

    fn parent_circuit(&self, circuit_instance: &Self::CellInstId) -> Self::CellId {
        self.circuit_inst(circuit_instance).parent_circuit_id
    }

    fn parent_circuit_of_pin(&self, pin: &Self::PinId) -> Self::CellId {
        self.pin(pin).circuit
    }

    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CellInstId {
        self.pin_inst(pin_inst).circuit_inst
    }


    /// Get the net connected to this pin.
    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.pin(pin).net
    }

    /// Get the net connected to this pin instance.
    fn net_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Option<Self::NetId> {
        self.pin_inst(pin_inst).net
    }

    fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.circuit(parent_circuit).net_low
    }

    fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.circuit(parent_circuit).net_high
    }

    fn net_by_name<N: ?Sized + Eq + Hash>(&self, parent_circuit: &Self::CellId, name: &N) -> Option<Self::NetId>
        where Self::NameType: Borrow<N> {
        self.circuit(parent_circuit).nets_by_name.get(name).copied()
    }

    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType> {
        self.net(net).name.clone()
    }

    fn circuit_name(&self, circuit: &Self::CellId) -> Self::NameType {
        self.circuit(circuit).name.clone()
    }

    fn circuit_instance_name(&self, circuit_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.circuit_inst(circuit_inst).name.clone()
    }

    fn for_each_circuit<F>(&self, f: F) where F: FnMut(Self::CellId) -> () {
        self.circuits.keys().copied().for_each(f)
    }

    /// Iterate over all circuits.
    fn each_circuit(&self) -> Box<dyn Iterator<Item=CellId> + '_> {
        Box::new(self.circuits.keys().copied())
    }

    fn for_each_instance<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.circuit(circuit).instances.iter()
            .copied().for_each(f)
    }

    fn each_instance(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.circuit(circuit).instances.iter().copied())
    }

    fn for_each_circuit_dependency<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.circuit(circuit).dependencies.keys().copied().for_each(f);
    }

    fn each_circuit_dependency(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuit(circuit).dependencies.keys().copied())
    }

    fn for_each_dependent_circuit<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.circuit(circuit).dependent_circuits.keys().copied().for_each(f);
    }

    fn each_dependent_circuit(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuit(circuit).dependent_circuits.keys().copied())
    }

    fn for_each_reference<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.circuit(circuit).references.iter().copied().for_each(f)
    }

    fn each_reference(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.circuit(circuit).references.iter().copied())
    }

    fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> () {
        self.circuit(circuit).pins.iter().copied().for_each(f)
    }

    /// Iterate over all pins of a circuit.
    fn each_pin(&self, circuit_id: &CellId) -> Box<dyn Iterator<Item=PinId> + '_> {
        Box::new(self.circuit(circuit_id).pins.iter().copied())
    }

    fn for_each_pin_instance<F>(&self, circuit_inst: &Self::CellInstId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.circuit_inst(circuit_inst).pins.iter().copied().for_each(f)
    }

    fn each_pin_instance<'a>(&'a self, circuit_inst: &Self::CellInstId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.circuit_inst(circuit_inst).pins.iter().copied())
    }

    fn for_each_internal_net<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::NetId) -> () {
        self.circuit(circuit).nets.iter().copied().for_each(f)
    }

    fn each_internal_net(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::NetId> + '_> {
        Box::new(self.circuit(circuit).nets.iter().copied())
    }

    fn num_child_instances(&self, circuit: &Self::CellId) -> usize {
        self.circuit(circuit).instances.len()
    }

    fn num_circuits(&self) -> usize {
        self.circuits.len()
    }

    fn num_pins(&self, circuit: &Self::CellId) -> usize {
        self.circuit(circuit).pins.len()
    }

    fn for_each_pin_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinId) -> () {
        self.net(net).pins.iter().copied().for_each(f)
    }

    fn each_pin_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinId> + 'a> {
        Box::new(self.net(net).pins.iter().copied())
    }

    fn for_each_pin_instance_of_net<F>(&self, net: &Self::NetId, f: F) where F: FnMut(Self::PinInstId) -> () {
        self.net(net).pin_instances.iter().copied().for_each(f)
    }

    fn each_pin_instance_of_net<'a>(&'a self, net: &Self::NetId) -> Box<dyn Iterator<Item=Self::PinInstId> + 'a> {
        Box::new(self.net(net).pin_instances.iter().copied())
    }
}

impl NetlistEdit for Chip {
    /// Create a new circuit with a given list of pins.
    fn create_circuit(&mut self, name: Self::NameType, pins: Vec<(Self::NameType, Direction)>) -> CellId {
        Chip::create_circuit(self, name, pins)
    }

    /// Remove all instances inside the circuit,
    fn remove_circuit(&mut self, circuit_id: &CellId) {
        Chip::remove_circuit(self, circuit_id)
    }

    /// Create a new instance of `circuit_template` in the `parent` circuit.
    fn create_circuit_instance(&mut self, parent: &CellId,
                               circuit_template: &CellId,
                               name: Option<Self::NameType>) -> CellInstId {
        Chip::create_circuit_instance(self, parent, circuit_template, name)
    }


    /// Remove a circuit instance after disconnecting it from the nets.
    fn remove_circuit_instance(&mut self, circuit_inst_id: &CellInstId) {
        Chip::remove_circuit_instance(self, circuit_inst_id)
    }

    /// Create a new net in the `parent` circuit.
    fn create_net(&mut self, parent: &CellId, name: Option<Self::NameType>) -> NetId {
        Chip::create_net(self, parent, name)
    }

    fn rename_net(&mut self, parent_circuit: &Self::CellId,
                  net_id: &Self::NetId, new_name: Option<Self::NameType>) {
        Chip::rename_net(self, parent_circuit, net_id, new_name)
    }

    /// Disconnect all connected terminals and remove the net.
    fn remove_net(&mut self, net: &NetId) {
        Chip::remove_net(self, net)
    }

    /// Connect the pin to a net.
    fn connect_pin(&mut self, pin: &PinId, net: Option<NetId>) -> Option<NetId> {
        Chip::connect_pin(self, pin, net)
    }

    /// Connect the pin to a net.
    fn connect_pin_instance(&mut self, pin: &PinInstId, net: Option<NetId>) -> Option<Self::NetId> {
        Chip::connect_pin_instance(self, pin, net)
    }
}

#[test]
fn test_create_populated_netlist() {
    let mut netlist = Chip::default();
    let top = netlist.create_circuit("TOP".into(),
                                     vec![
                                         ("A".into(), Direction::Input),
                                         ("B".into(), Direction::Output)
                                     ],
    );
    assert_eq!(Some(top), netlist.circuit_by_name("TOP"));

    let sub_a = netlist.create_circuit("SUB_A".into(),
                                       vec![
                                           ("A".into(), Direction::Input),
                                           ("B".into(), Direction::Output)
                                       ],
    );
    let sub_b = netlist.create_circuit("SUB_B".into(),
                                       vec![
                                           ("A".into(), Direction::Input),
                                           ("B".into(), Direction::Output)
                                       ],
    );

    let inst_a = netlist.create_circuit_instance(&top, &sub_a, None);
    let _inst_b = netlist.create_circuit_instance(&top, &sub_b, None);

    let net_a = netlist.create_net(&top, Some("NetA".into()));
    let net_b = netlist.create_net(&top, Some("NetB".into()));

    let pins_a = netlist.each_pin_instance(&inst_a).collect_vec();
    let pins_top = netlist.each_pin(&top).collect_vec();

    netlist.connect_pin_instance(&pins_a[0], Some(net_a));
    netlist.connect_pin_instance(&pins_a[1], Some(net_b));

    netlist.connect_pin(&pins_top[0], Some(net_a));
    netlist.connect_pin(&pins_top[1], Some(net_b));

    dbg!(&netlist);
    dbg!(netlist.terminals_for_net(&net_a).collect_vec());

    assert_eq!(netlist.num_net_terminals(&net_a), 2);
    assert_eq!(netlist.num_net_terminals(&net_b), 2);
}


/// Wrapper around a `Geometry` struct.
#[derive(Clone, Debug)]
pub struct Shape<C: CoordinateType, U = ()> {
    /// Identifier of this shape.
    index: Index<Self>,
    /// The geometry of this shape.
    pub geometry: Geometry<C>,
    // /// Reference ID to container.
    // parent_id: Index<Shapes<T>>,
    /// User-defined data.
    user_data: U,
}

/// `Shapes<T>` is a collection of `Shape<T>` structs. Each of
/// the elements is assigned an index when inserted into the collection.
#[derive(Clone, Debug)]
pub struct Shapes<C>
    where C: CoordinateType {
    /// ID of this shape collection.
    id: Index<Self>,
    /// Reference to the cell where this shape collection lives. Can be none.
    parent_cell: CellId,
    /// Shape elements.
    shapes: IntHashMap<ShapeId, Shape<C>>,
    /// Property stores for the shapes.
    shape_properties: IntHashMap<ShapeId, PropertyStore<RcString>>,
}

impl<C: CoordinateType> Shapes<C> {
    /// Get the ID of this shape container.
    pub fn id(&self) -> Index<Self> {
        self.id
    }

    /// Iterate over all geometric shapes in this collection.
    pub fn each_shape(&self) -> impl Iterator<Item=&Shape<C>> {
        self.shapes.values()
    }
}



impl HierarchyBase for Chip<Coord> {
    type NameType = RcString;
    type CellId = CellId;
    type CellInstId = CellInstId;

    /// Create a new empty layout.
    fn new() -> Self {
        Chip::default()
    }

    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N> {
        Chip::circuit_by_name(self, name)
    }

    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuits.keys().copied())
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        self.circuit(cell).name().clone()
    }

    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.circuit_inst(cell_inst).name.clone()
    }

    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.circuit(cell).each_instance_id())
    }

    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuit(cell).each_dependent_cell_id())
    }

    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuit(cell).each_dependency_id())
    }

    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.circuit_inst(cell_instance).parent_circuit_id()
    }

    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.circuit_inst(cell_instance).template_circuit_id()
    }

}

impl LayoutBase for Chip<Coord> {
    type Coord = Coord;
    type LayerId = LayerId;
    type ShapeId = ShapeId;

    fn dbu(&self) -> Self::Coord {
        self.dbu
    }

    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_> {
        Box::new(self.layer_info.keys().copied())
    }

    fn layer_info(&self, layer: &Self::LayerId) -> &LayerInfo {
        &self.layer_info[layer]
    }


    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.layers_by_index_datatype.get(&(index, datatype)).copied()
    }

    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        Box::new(self.circuit(cell).shapes(layer).expect("Layer not found.")
            .each_shape().map(|s| s.index))
    }

    // fn each_shape(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=&Geometry<Self::Coord>> + '_> {
    //     Box::new(self.cells[cell].shapes_map[layer].shapes.values().map(|s| &s.geometry))
    // }

    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, mut f: F)
        where F: FnMut(&Geometry<Self::Coord>) -> () {
        self.circuit(cell).shapes_map[layer].shapes.values()
            .for_each(|s| f(&s.geometry))
    }
}

impl LayoutEdit for Chip<Coord> {
    fn find_or_create_layer(&mut self, index: u32, datatype: u32) -> Self::LayerId {
        let layer = self.find_layer(index, datatype);
        if let Some(layer) = layer {
            layer
        } else {
            // Find next free layer index.
            let layer_index = self.layer_index_generator.next();
            // Create new entries in the layer lookup tables.
            self.layers_by_index_datatype.insert((index, datatype), layer_index);

            let info = LayerInfo { index, datatype, name: None };
            self.layer_info.insert(layer_index, info);
            layer_index
        }
    }

    fn create_cell(&mut self, name: RcString) -> Self::CellId {
        Chip::create_circuit(self, name, vec![])
    }

    fn remove_cell(&mut self, cell_id: &Self::CellId) {
        Chip::remove_circuit(self, cell_id)
    }

    fn create_cell_instance(&mut self, parent_cell: &Self::CellId,
                            template_cell: &Self::CellId,
                            name: Option<RcString>,
                            transform: SimpleTransform<Self::Coord>) -> Self::CellInstId {
        let id = <Chip<Self::Coord>>::create_circuit_instance(self, parent_cell, template_cell, name);
        self.circuit_inst_mut(&id).set_transform(transform);
        id
    }

    fn remove_cell_instance(&mut self, id: &Self::CellInstId) {
        <Chip<Self::Coord>>::remove_circuit_instance(self, id)
    }

    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId{
        let shape_id = self.shape_index_generator.next();

        let shape = Shape {
            index: shape_id,
            geometry,
            user_data: Default::default(),
        };

        self.circuit_mut(parent_cell)
            .shapes_mut(layer).expect("Layer not found.")
            .shapes
            .insert(shape_id, shape);

        shape_id
    }

    fn remove_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, shape_id: &Self::ShapeId)
                    -> Option<Geometry<Self::Coord>> {
        self.circuit_mut(parent_cell)
            .shapes_mut(layer).expect("Layer not found.")
            .shapes.remove(shape_id)
            .map(|s| s.geometry)
    }

    fn replace_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId,
                     shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>)
                     -> Option<Geometry<Self::Coord>> {
        let shape_id = *shape_id;
        let shape = Shape {
            index: shape_id,
            geometry,
            user_data: Default::default(),
        };

        self.circuit_mut(parent_cell)
            .shapes_mut(layer).expect("Layer not found.")
            .shapes.insert(shape_id, shape)
            .map(|s| s.geometry)
    }
}