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
//! [`Chip`] implements the [`L2NEdit`] trait and hence qualifies for representing
//! netlists fused with a layout. [`Chip`] can also be used solely as a netlist structure
//! or as a layout structure.

// TODO: Remove this when fully implemented.
#![allow(unused_variables)]

use iron_shapes::CoordinateType;
use iron_shapes::prelude::{Rect, Geometry};
use iron_shapes::transform::SimpleTransform;

use crate::index::*;
use std::collections::HashMap;
use itertools::Itertools;
use std::borrow::{Borrow, BorrowMut};
use std::hash::Hash;
use crate::prelude::{
    HierarchyBase, HierarchyEdit,
    NetlistBase, NetlistEdit,
    LayoutBase, LayoutEdit,
    L2NBase, L2NEdit,
    MapPointwise,
};

use crate::netlist::direction::Direction;
// use crate::rc_string::RcString;
use std::fmt::Debug;

use crate::property_storage::{PropertyStore, PropertyValue};
use crate::layout::types::{LayerInfo};

// Use an alternative hasher that has better performance for integer keys.
use fnv::{FnvHashMap, FnvHashSet};

use crate::prelude::{TryBoundingBox};
use num_traits::One;

type NameT = String;

type IntHashMap<K, V> = FnvHashMap<K, V>;
type IntHashSet<V> = FnvHashSet<V>;

/// Default unsigned integer type.
pub type UInt = u32;
/// Default signed integer type.
pub type SInt = i32;

/// Integer coordinate type.
pub type Coord = i32;
/// Integer area type.
pub type Area = i64;

/// Circuit identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CellId(u32);

/// Circuit instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CellInstId(usize);

/// Pin identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PinId(u32);

/// Pin instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PinInstId(usize);

/// Either a pin or pin instance identifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NetId(usize);

/// Unique (across layout) identifier of a shape.
pub type ShapeId = Index<Shape<Coord>, u32>;

/// ID for layers.
pub type LayerId = Index<LayerInfo<NameT>, u16>;

/// Allow creating IDs from integers.
macro_rules! impl_from_for_id {
    ($t:tt, $i:ty) => {
        impl From<$i> for $t {
            fn from(id: $i) -> Self {
                $t(id)
            }
        }
    }
}

impl_from_for_id!(CellId, u32);
impl_from_for_id!(CellInstId, usize);
impl_from_for_id!(PinId, u32);
impl_from_for_id!(PinInstId, usize);
impl_from_for_id!(NetId, usize);


/// A circuit is defined by an interface (pins) and
/// a content which consists of interconnected circuit instances.
///
/// Template parameters:
///
/// * `U`: User defined data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Circuit<C = Coord, U = ()>
    where C: CoordinateType, U: Default {
    /// ID of this circuit.
    id: CellId,
    /// Name of the circuit.
    name: NameT,

    /// Instances inside this circuit.
    instances: IntHashSet<CellInstId>,
    /// Instances inside this circuit indexed by name.
    /// Not every instance needs to have a name.
    instances_by_name: HashMap<NameT, CellInstId>,
    /// Circuit instances that reference to this circuit.
    references: IntHashSet<CellInstId>,

    /// Set of circuits that are direct dependencies of this circuit.
    /// Stored together with a counter of how many instances of the dependency are present.
    /// This are the circuits towards the leaves in the dependency tree.
    dependencies: IntHashMap<CellId, usize>,
    /// Circuits that use a instance of this circuit.
    dependent_circuits: IntHashMap<CellId, usize>,

    /// Properties related to the instances in this template.
    /// Instance properties are stored here for lower overhead of cell instances.
    #[allow(unused)]
    instance_properties: IntHashMap<CellInstId, PropertyStore<NameT>>,
    /// Properties related to this template.
    properties: PropertyStore<NameT>,
    /// User-defined data.
    #[allow(unused)]
    user_data: U,

    // == Netlist == //

    /// Pin definitions, the actual pin structs are in the top level `Chip` struct.
    pins: Vec<PinId>,
    /// All nets in this circuit.
    nets: IntHashSet<NetId>,
    /// Nets IDs stored by name.
    nets_by_name: HashMap<NameT, NetId>,
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
    fn id(&self) -> CellId {
        self.id
    }

    // == Layout == //

    /// Get the shape container of this layer.
    /// Returns `None` if the shapes object does not exist for this layer.
    fn shapes(&self, layer_id: &LayerId) -> Option<&Shapes<Coord>> {
        self.shapes_map.get(layer_id)
    }

    /// Get the mutable shape container of this layer.
    /// Returns `None` if the shapes object does not exist for this layer.
    fn shapes_mut(&mut self, layer_id: &LayerId) -> Option<&mut Shapes<Coord>> {
        self.shapes_map.get_mut(layer_id)
    }

    /// Get a mutable reference to the shapes container of the `layer`. When none exists
    /// a shapes object is created for this layer.
    fn get_or_create_shapes_mut(&mut self, layer_id: &LayerId) -> &mut Shapes<Coord> {
        let self_id = self.id();
        self.shapes_map.entry(*layer_id)
            .or_insert_with(|| Shapes::new())
            .borrow_mut()
    }
}

// pub enum PlacementStatus {
//     Unplaced,
//     Fixed
// }

/// Instance of a circuit.
///
/// Template parameters:
///
/// * `U`: User defined data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CircuitInst<C = Coord, U = ()>
    where C: CoordinateType {
    /// Name of the instance.
    name: Option<NameT>,
    /// The ID of the template circuit.
    template_circuit_id: CellId,
    /// The ID of the parent circuit where this instance lives in.
    parent_circuit_id: CellId,
    /// Properties related to this instance.
    properties: PropertyStore<NameT>,

    /// User-defined data.
    #[allow(unused)]
    user_data: U,

    // == Netlist == //

    /// List of pins of this instance.
    pins: Vec<PinInstId>,

    // == Layout == //
    /// Transformation to put the cell to the right place an into the right scale/rotation.
    transform: SimpleTransform<C>,
    // TODO: Repetition
    // /// Current status of the cell placement.
    // placement_status: PlacementStatus
}

impl CircuitInst {
    // == Layout == //

    /// Get the transformation that represents the location and orientation of this instance.
    fn get_transform(&self) -> &SimpleTransform<Coord> {
        &self.transform
    }

    /// Set the transformation that represents the location and orientation of this instance.
    fn set_transform(&mut self, tf: SimpleTransform<Coord>) {
        self.transform = tf;
    }
}

/// Single bit wire pin.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pin {
    /// Name of the pin.
    name: NameT,
    /// Signal type/direction of the pin.
    direction: Direction,
    /// Parent circuit of this pin.
    circuit: CellId,
    /// Net that is connected to this pin.
    net: Option<NetId>,

    // == Layout == //
    /// List of shapes in the layout that represent the physical pin.
    pin_shapes: IntHashSet<ShapeId>,
}

/// Instance of a pin.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PinInst {
    /// ID of the template pin.
    pub template_pin_id: PinId,
    /// Circuit instance where this pin instance lives in.
    pub circuit_inst: CellInstId,
    /// Net connected to this pin instance.
    net: Option<NetId>,
}

/// A net represents an electric potential or a wire.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Net {
    /// Name of the net.
    pub name: Option<NameT>,
    /// Parent circuit of the net.
    pub parent_id: CellId,
    /// Pins connected to this net.
    pub pins: IntHashSet<PinId>,
    /// Pin instances connected to this net.
    pub pin_instances: IntHashSet<PinInstId>,

    // == Layout == //

    /// List of shapes in the layout that represent the physical net.
    pub net_shapes: IntHashSet<ShapeId>,
}

impl Net {}

/// A netlist is the container of circuits.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chip<C: CoordinateType = Coord> {
    circuits: IntHashMap<CellId, Circuit<C>>,
    circuits_by_name: HashMap<NameT, CellId>,
    circuit_instances: IntHashMap<CellInstId, CircuitInst>,
    nets: IntHashMap<NetId, Net>,
    pins: IntHashMap<PinId, Pin>,
    pin_instances: IntHashMap<PinInstId, PinInst>,

    /// Top-level properties.
    properties: PropertyStore<NameT>,

    id_counter_circuit: u32,
    id_counter_circuit_inst: usize,
    id_counter_pin: u32,
    id_counter_pin_inst: usize,
    id_counter_net: usize,

    // == Layout == //

    dbu: C,

    /// Counter for generating the next layer index.
    layer_index_generator: IndexGenerator<LayerInfo<NameT>, u16>,
    /// Lookup table for finding layers by name.
    layers_by_name: HashMap<NameT, LayerId>,
    /// Lookup table for finding layers by index/datatype numbers.
    layers_by_index_datatype: IntHashMap<(UInt, UInt), LayerId>,
    /// Info structures for all layers.
    layer_info: IntHashMap<LayerId, LayerInfo<NameT>>,
    /// ID generator for shapes.
    shape_index_generator: IndexGenerator<Shape<C>>,

    /// Link to the cell and layer that contain a shape.
    shape_parents: IntHashMap<ShapeId, (CellId, LayerId)>,

    /// Link to the shapes of a net.
    net_shapes: IntHashMap<NetId, IntHashSet<ShapeId>>,
}

impl<C: CoordinateType + One> Default for Chip<C> {
    fn default() -> Self {
        Self {
            circuits: Default::default(),
            circuits_by_name: Default::default(),
            circuit_instances: Default::default(),
            nets: Default::default(),
            pins: Default::default(),
            pin_instances: Default::default(),
            properties: Default::default(),
            id_counter_circuit: 0,
            id_counter_circuit_inst: 0,
            id_counter_pin: 0,
            id_counter_pin_inst: 0,
            id_counter_net: 0,
            dbu: C::one(),
            layer_index_generator: Default::default(),
            layers_by_name: Default::default(),
            layers_by_index_datatype: Default::default(),
            layer_info: Default::default(),
            shape_index_generator: Default::default(),
            shape_parents: Default::default(),
            net_shapes: Default::default(),
        }
    }
}

impl Chip<Coord> {
    /// Find a circuit by its name.
    fn circuit_by_name<S: ?Sized + Eq + Hash>(&self, name: &S) -> Option<CellId>
        where NameT: Borrow<S> {
        self.circuits_by_name.get(name).copied()
    }

    /// Change the name of the cell.
    ///
    /// # Panics
    /// Panics if the name already exists.
    fn rename_cell(&mut self, cell: &CellId, name: NameT) {
        assert!(!self.circuits_by_name.contains_key(&name),
                "Cell with this name already exists: {}", &name);

        // Remove old name.
        let old_name = &self.circuits[cell].name;
        let id = self.circuits_by_name.remove(old_name);
        debug_assert_eq!(id.as_ref(), Some(cell));

        // Set the new name.
        self.circuit_mut(cell).name = name.clone();
        self.circuits_by_name.insert(name, cell.clone());
    }

    /// Change the name of the cell instance.
    ///
    /// # Panics
    /// Panics if the name already exists.
    fn rename_cell_instance(&mut self, inst: &CellInstId, name: Option<NameT>) {
        let parent = self.parent_cell(inst);
        if let Some(name) = &name {
            assert!(!self.circuit(&parent).instances_by_name.contains_key(name),
                    "Cell with this name already exists: {}", name);
        }

        // Remove old name.
        let old_name = self.circuit_inst_mut(inst).name.take();
        if let Some(old_name) = old_name {
            self.circuit_mut(&parent).instances_by_name.remove(&old_name);
        }

        self.circuit_inst_mut(inst).name = name.clone();
        if let Some(name) = name {
            self.circuit_mut(&parent).instances_by_name.insert(name, inst.clone());
        }
    }

    /// Create a new circuit template.
    fn create_circuit(&mut self, name: NameT, pins: Vec<(NameT, Direction)>) -> CellId {
        assert!(!self.circuits_by_name.contains_key(&name),
                "Circuit with this name already exists: {}", &name);
        let id = CellId(Self::next_id_counter_u32(&mut self.id_counter_circuit));

        let circuit = Circuit {
            id,
            name: name.clone(),
            pins: Default::default(),
            instances: Default::default(),
            instances_by_name: Default::default(),
            references: Default::default(),
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
            properties: Default::default(),
        };

        self.circuits.insert(id, circuit);
        self.circuits_by_name.insert(name, id);

        // Create LOW and HIGH nets.
        let net_low = self.create_net(&id, Some("__LOW__".into()));
        let net_high = self.create_net(&id, Some("__HIGH__".into()));

        let c = self.circuit_mut(&id);
        c.net_low = net_low;
        c.net_high = net_high;

        // Create pins.
        pins.into_iter()
            .for_each(|(name, direction)| {
                self.create_pin(id, name, direction);
            });

        id
    }

    /// Remove all instances inside the circuit, remove all instances of the circuit
    /// and remove finally the circuit itself.
    fn remove_circuit(&mut self, circuit_id: &CellId) {
        // Remove all instances inside this circuit.
        let instances = self.circuit(circuit_id).instances.iter().copied().collect_vec();
        for inst in instances {
            self.remove_circuit_instance(&inst);
        }

        // Clean up links to cell shapes.
        for shape_id in self.circuits[&circuit_id]
            .shapes_map.iter()
            .flat_map(|(layer, shapes)| shapes.shapes.keys()) {
            self.shape_parents.remove(shape_id);
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
    fn create_circuit_instance(&mut self, parent: &CellId,
                               circuit_template: &CellId,
                               name: Option<NameT>) -> CellInstId {
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
            properties: Default::default(),
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
    fn remove_circuit_instance(&mut self, circuit_inst_id: &CellInstId) {

        // Remove the instance name.
        self.rename_cell_instance(circuit_inst_id, None);

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
    fn create_net(&mut self, parent: &CellId, name: Option<NameT>) -> NetId {
        assert!(self.circuits.contains_key(parent));

        let id = NetId(Self::next_id_counter_usize(&mut self.id_counter_net));
        let net = Net {
            name: name.clone(),
            parent_id: *parent,
            pins: Default::default(),
            pin_instances: Default::default(),
            net_shapes: Default::default(),
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

    /// Change the name of the net.
    fn rename_net(&mut self, net_id: &NetId, new_name: Option<NameT>) -> Option<NameT> {
        let parent_circuit = self.parent_cell_of_net(net_id);

        // Check if a net with this name already exists.
        if let Some(name) = &new_name {
            if let Some(other) = self.circuit(&parent_circuit).nets_by_name.get(name) {
                if other != net_id {
                    panic!("Net name already exists.")
                } else {
                    // Name is the same as before.
                    return new_name;
                }
            }
        }

        let maybe_old_name = self.net_mut(net_id).name.take();

        // Remove the old name mapping.
        if let Some(old_name) = &maybe_old_name {
            self.circuit_mut(&parent_circuit).nets_by_name.remove(old_name);
        }

        // Add the new name mapping.
        if let Some(new_name) = new_name {
            self.nets.get_mut(net_id)
                .expect("Net not found.")
                .name.replace(new_name.clone());
            self.circuit_mut(&parent_circuit).nets_by_name.insert(new_name, *net_id);
        }

        maybe_old_name
    }

    /// Disconnect all connected terminals and remove the net.
    fn remove_net(&mut self, net: &NetId) {
        let parent_circuit = self.net(net).parent_id;

        assert_ne!(net, &self.net_zero(&parent_circuit), "Cannot remove constant LOW net.");
        assert_ne!(net, &self.net_one(&parent_circuit), "Cannot remove constant HIGH net.");

        // Remove all links from shapes to this net.
        let net_shapes = self.net_shapes.get(net)
            .iter()
            .flat_map(|shape_ids| shape_ids.iter().cloned())
            .collect_vec();
        for net_shape in &net_shapes {
            self.set_net_of_shape(net_shape, None);
        }

        // Remove all links to pins.
        let pins = self.pins_for_net(net).collect_vec();
        let pin_insts = self.pins_instances_for_net(net).collect_vec();

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
    fn disconnect_pin(&mut self, pin: &PinId) -> Option<NetId> {
        self.connect_pin(pin, None)
    }

    /// Connect the pin to a net.
    fn connect_pin(&mut self, pin: &PinId, net: Option<NetId>) -> Option<NetId> {
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
    fn disconnect_pin_instance(&mut self, pin: &PinInstId) -> Option<NetId> {
        self.connect_pin_instance(pin, None)
    }

    /// Connect the pin to a net.
    fn connect_pin_instance(&mut self, pin: &PinInstId, net: Option<NetId>) -> Option<NetId> {
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
    fn circuit(&self, id: &CellId) -> &Circuit {
        &self.circuits[id]
    }

    // /// Get a fat circuit reference by its ID.
    // fn circuit_ref(&self, id: &CellId) -> CircuitRef {
    //     CircuitRef {
    //         netlist: self,
    //         circuit: &self.circuits[id],
    //     }
    // }

    /// Get a mutable reference to the circuit by its ID.
    fn circuit_mut(&mut self, id: &CellId) -> &mut Circuit {
        self.circuits.get_mut(id).expect("Cell ID not found.")
    }

    /// Get a reference to a circuit instance.
    fn circuit_inst(&self, id: &CellInstId) -> &CircuitInst {
        &self.circuit_instances[id]
    }

    /// Get a mutable reference to a circuit instance.
    fn circuit_inst_mut(&mut self, id: &CellInstId) -> &mut CircuitInst {
        self.circuit_instances.get_mut(id).unwrap()
    }

    /// Get a reference to a net by its ID.
    fn net(&self, id: &NetId) -> &Net {
        &self.nets.get(id).expect("Net ID does not exist in this netlist.")
    }

    /// Get a mutable reference to a net by its ID.
    fn net_mut(&mut self, id: &NetId) -> &mut Net {
        self.nets.get_mut(id).unwrap()
    }

    /// Get a reference to a pin by its ID.
    fn pin(&self, id: &PinId) -> &Pin {
        &self.pins[id]
    }

    /// Get a mutable reference to a pin by its ID.
    fn pin_mut(&mut self, id: &PinId) -> &mut Pin {
        self.pins.get_mut(id).unwrap()
    }

    /// Get a reference to a pin instance by its ID.
    fn pin_inst(&self, id: &PinInstId) -> &PinInst {
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
    /// Update all circuit instances with the new pin.
    fn create_pin(&mut self, parent: CellId, name: NameT, direction: Direction) -> PinId {
        let pin_id = PinId(Self::next_id_counter_u32(&mut self.id_counter_pin));
        let pin = Pin {
            name,
            direction,
            circuit: parent,
            net: Default::default(),
            pin_shapes: Default::default(),
        };
        self.pins.insert(pin_id, pin);

        // Register the pin in the circuit.
        self.circuits.get_mut(&parent).unwrap()
            .pins.push(pin_id);

        // Insert the pin in all instances of this circuit.
        for inst in &self.circuits[&parent].references {

            // Create new pin instance.
            let pin_inst_id = PinInstId(Self::next_id_counter_usize(&mut self.id_counter_pin_inst));
            let pin = PinInst {
                template_pin_id: pin_id,
                circuit_inst: *inst,
                net: None,
            };
            self.pin_instances.insert(pin_inst_id, pin);

            // Register the pin instance in the circuit instance.
            self.circuit_instances.get_mut(inst).unwrap().pins
                .push(pin_inst_id);
        }

        pin_id
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

    /// Iterate over all pins connected to a net.
    fn pins_for_net(&self, net: &NetId) -> impl Iterator<Item=PinId> + '_ {
        self.net(net).pins.iter().copied()
    }

    /// Iterate over all pin instances connected to a net.
    fn pins_instances_for_net(&self, net: &NetId) -> impl Iterator<Item=PinInstId> + '_ {
        self.net(net).pin_instances.iter().copied()
    }

    /// Get a mutable reference to a shape struct by its ID.
    fn shape_mut(&mut self, shape_id: &ShapeId) -> &mut Shape<Coord> {
        let (cell, layer) = self.shape_parents.get(shape_id)
            .expect("Shape not found.").clone();
        self.circuit_mut(&cell)
            .shapes_mut(&layer)
            .expect("Layer not found.")
            .shapes.get_mut(shape_id)
            .expect("Shape not found.")
    }

    /// Get a reference to a shape struct by its ID.
    fn shape(&self, shape_id: &ShapeId) -> &Shape<Coord> {
        let (cell, layer) = self.shape_parents.get(shape_id)
            .expect("Shape not found.").clone();
        self.circuit(&cell)
            .shapes(&layer)
            .expect("Layer not found.")
            .shapes.get(shape_id)
            .expect("Shape not found.")
    }
}


impl NetlistBase for Chip {
    type PinId = PinId;
    type PinInstId = PinInstId;
    type NetId = NetId;


    fn template_pin(&self, pin_instance: &Self::PinInstId) -> Self::PinId {
        self.pin_inst(pin_instance).template_pin_id
    }

    fn pin_direction(&self, pin: &Self::PinId) -> Direction {
        self.pin(pin).direction
    }

    fn pin_name(&self, pin: &Self::PinId) -> Self::NameType {
        self.pin(pin).name.clone()
    }

    fn pin_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::PinId>
    {
        // TODO: Create index for pin names.
        self.circuit(&parent_circuit).pins.iter()
            .find(|p| self.pin(*p).name.as_str() == name)
            .copied()
    }

    fn parent_cell_of_pin(&self, pin: &Self::PinId) -> Self::CellId {
        self.pin(pin).circuit
    }

    fn parent_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Self::CellInstId {
        self.pin_inst(pin_inst).circuit_inst
    }

    fn parent_cell_of_net(&self, net: &Self::NetId) -> Self::CellId {
        self.nets[net].parent_id
    }

    fn net_of_pin(&self, pin: &Self::PinId) -> Option<Self::NetId> {
        self.pin(pin).net
    }

    fn net_of_pin_instance(&self, pin_inst: &Self::PinInstId) -> Option<Self::NetId> {
        self.pin_inst(pin_inst).net
    }

    fn net_zero(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.circuit(parent_circuit).net_low
    }

    fn net_one(&self, parent_circuit: &Self::CellId) -> Self::NetId {
        self.circuit(parent_circuit).net_high
    }

    fn net_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::NetId> {
        self.circuit(parent_circuit).nets_by_name.get(name).copied()
    }

    fn net_name(&self, net: &Self::NetId) -> Option<Self::NameType> {
        self.net(net).name.clone()
    }

    fn for_each_pin<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::PinId) -> () {
        self.circuit(circuit).pins.iter().copied().for_each(f)
    }

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

    fn num_internal_nets(&self, circuit: &Self::CellId) -> usize {
        self.circuit(circuit).nets.len()
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
    fn create_pin(&mut self, circuit: &Self::CellId, name: Self::NameType, direction: Direction) -> Self::PinId {
        Chip::create_pin(self, *circuit, name, direction)
    }

    fn remove_pin(&mut self, id: &Self::PinId) {

        // Remove all links from shapes to this pin.
        let pin_shapes = self.pin(id).pin_shapes
            .iter()
            .cloned()
            .collect_vec();
        for pin_shape in &pin_shapes {
            self.set_pin_of_shape(pin_shape, None);
        }

        // Disconnect the pin for all instances.
        let cell = self.parent_cell_of_pin(id);
        for inst in self.each_cell_instance_vec(&cell) {
            let pin_inst = self.pin_instance(&inst, id);
            self.disconnect_pin_instance(&pin_inst);

            // Remove pin instance.
            self.circuit_inst_mut(&inst)
                .pins.retain(|p| p != &pin_inst);
            // Delete the pin instance struct.
            self.pin_instances.remove(&pin_inst);
        }

        // Disconnect this pin from internal nets.
        self.disconnect_pin(id);

        // Remove the pin from the cell.
        self.circuit_mut(&cell).pins.retain(|p| p != id);
        // Delete the pin struct.
        self.pins.remove(id);
    }

    fn rename_pin(&mut self, pin: &Self::PinId, new_name: Self::NameType) -> Self::NameType {
        let cell = self.parent_cell_of_pin(&pin);
        let existing = self.pin_by_name(&cell, &new_name);
        if existing.is_some() {
            panic!("Pin name already exists in cell '{}': '{}'", self.cell_name(&cell), new_name)
        }
        let old_name = std::mem::replace(&mut self.pin_mut(pin).name, new_name);
        old_name
    }

    fn create_net(&mut self, parent: &CellId, name: Option<Self::NameType>) -> NetId {
        Chip::create_net(self, parent, name)
    }

    fn rename_net(&mut self, net_id: &Self::NetId, new_name: Option<Self::NameType>) -> Option<Self::NameType> {
        Chip::rename_net(self, net_id, new_name)
    }

    fn remove_net(&mut self, net: &NetId) {
        Chip::remove_net(self, net)
    }

    fn connect_pin(&mut self, pin: &PinId, net: Option<NetId>) -> Option<NetId> {
        Chip::connect_pin(self, pin, net)
    }

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
                                     ],
    );
    netlist.create_pin(top, "B".into(), Direction::Output);
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

    assert_eq!(netlist.num_net_terminals(&net_a), 2);
    assert_eq!(netlist.num_net_terminals(&net_b), 2);
}


/// Wrapper around a `Geometry` struct.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Shape<C, U = ()> {
    /// Identifier of this shape.
    index: Index<Shape<C, U>>,
    /// The geometry of this shape.
    pub geometry: Geometry<C>,
    // /// Reference ID to container.
    // parent_id: Index<Shapes<T>>,
    /// Net attached to this shape.
    net: Option<NetId>,
    /// Pin that belongs to this shape.
    pin: Option<PinId>,
    /// User-defined data.
    #[allow(unused)]
    user_data: U,
}

/// `Shapes<T>` is a collection of `Shape<T>` structs. Each of
/// the elements is assigned an index when inserted into the collection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Shapes<C>
    where C: CoordinateType {
    /// Shape elements.
    shapes: IntHashMap<ShapeId, Shape<C>>,
    /// Property stores for the shapes.
    shape_properties: IntHashMap<ShapeId, PropertyStore<NameT>>,
}

impl<C: CoordinateType> Shapes<C> {
    /// Create a new empty shapes container.
    fn new() -> Self {
        Self {
            shapes: Default::default(),
            shape_properties: Default::default(),
        }
    }
    //
    // /// Get the ID of this shape container.
    // fn id(&self) -> Index<Self> {
    //     self.id
    // }

    /// Iterate over all shapes in this container.
    fn each_shape(&self) -> impl Iterator<Item=&Shape<C>> {
        self.shapes.values()
    }
}

impl<C: CoordinateType> TryBoundingBox<C> for Shapes<C> {
    fn try_bounding_box(&self) -> Option<Rect<C>> {
        // Compute the bounding box from scratch.

        self.each_shape()
            .fold(None, |bbox, shape| {
                let bbox_new = shape.geometry.try_bounding_box();
                match bbox {
                    None => bbox_new,
                    Some(bbox1) => {
                        Some(
                            // Compute bounding box of the two rectangles.
                            match bbox_new {
                                None => bbox1,
                                Some(bbox2) => bbox1.add_rect(&bbox2)
                            }
                        )
                    }
                }
            })
    }
}

impl HierarchyBase for Chip<Coord> {
    type NameType = NameT;
    type CellId = CellId;
    type CellInstId = CellInstId;

    fn cell_by_name(&self, name: &str) -> Option<CellId> {
        Chip::circuit_by_name(self, name)
    }

    fn cell_instance_by_name(&self, parent_circuit: &Self::CellId, name: &str) -> Option<Self::CellInstId>
    {
        self.circuit(parent_circuit).instances_by_name.get(name).copied()
    }

    fn cell_name(&self, circuit: &Self::CellId) -> Self::NameType {
        self.circuit(circuit).name.clone()
    }

    fn cell_instance_name(&self, circuit_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.circuit_inst(circuit_inst).name.clone()
    }

    fn parent_cell(&self, circuit_instance: &Self::CellInstId) -> Self::CellId {
        self.circuit_inst(circuit_instance).parent_circuit_id
    }

    fn template_cell(&self, circuit_instance: &Self::CellInstId) -> Self::CellId {
        self.circuit_inst(circuit_instance).template_circuit_id
    }

    fn for_each_cell<F>(&self, f: F) where F: FnMut(Self::CellId) -> () {
        self.circuits.keys().copied().for_each(f)
    }

    fn each_cell(&self) -> Box<dyn Iterator<Item=CellId> + '_> {
        Box::new(self.circuits.keys().copied())
    }

    fn for_each_cell_instance<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.circuit(circuit).instances.iter()
            .copied().for_each(f)
    }

    fn each_cell_instance(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.circuit(circuit).instances.iter().copied())
    }

    fn for_each_cell_dependency<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.circuit(circuit).dependencies.keys().copied().for_each(f);
    }

    fn each_cell_dependency(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuit(circuit).dependencies.keys().copied())
    }

    fn num_cell_dependencies(&self, cell: &Self::CellId) -> usize {
        self.circuit(cell).dependencies.len()
    }

    fn for_each_dependent_cell<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.circuit(circuit).dependent_circuits.keys().copied().for_each(f);
    }

    fn each_dependent_cell(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        Box::new(self.circuit(circuit).dependent_circuits.keys().copied())
    }

    fn num_dependent_cells(&self, cell: &Self::CellId) -> usize {
        self.circuit(cell).dependent_circuits.len()
    }

    fn for_each_cell_reference<F>(&self, circuit: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.circuit(circuit).references.iter().copied().for_each(f)
    }

    fn each_cell_reference(&self, circuit: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        Box::new(self.circuit(circuit).references.iter().copied())
    }

    fn num_cell_references(&self, cell: &Self::CellId) -> usize {
        self.circuit(cell).references.len()
    }

    fn num_child_instances(&self, cell: &Self::CellId) -> usize {
        self.circuit(cell).instances.len()
    }

    fn num_cells(&self) -> usize {
        self.circuits.len()
    }


    fn get_chip_property(&self, key: &Self::NameType) -> Option<PropertyValue> {
        self.properties.get(key).cloned()
    }

    fn get_cell_property(&self, cell: &Self::CellId, key: &Self::NameType) -> Option<PropertyValue> {
        self.circuit(cell).properties.get(key).cloned()
    }

    fn get_cell_instance_property(&self, inst: &Self::CellInstId, key: &Self::NameType) -> Option<PropertyValue> {
        self.circuit_inst(inst).properties.get(key).cloned()
    }
}

impl LayoutBase for Chip<Coord> {
    type Coord = Coord;
    type Area = Area;
    type LayerId = LayerId;
    type ShapeId = ShapeId;

    fn dbu(&self) -> Self::Coord {
        self.dbu
    }

    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_> {
        Box::new(self.layer_info.keys().copied())
    }

    fn layer_info(&self, layer: &Self::LayerId) -> LayerInfo<Self::NameType> {
        self.layer_info[layer].clone()
    }

    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.layers_by_index_datatype.get(&(index, datatype)).copied()
    }

    fn layer_by_name(&self, name: &str) -> Option<Self::LayerId> {
        self.layers_by_name.get(name).cloned()
    }

    fn bounding_box_per_layer(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Option<Rect<Coord>> {
        // Compute the bounding box of the cell's own shapes.
        let mut bbox = self.circuit(cell)
            .shapes(layer)
            .and_then(|shapes| shapes.try_bounding_box());
        // Update the bounding box with the children cells.
        self.for_each_cell_instance(cell, |i| {
            let template = self.template_cell(&i);
            let tf = self.get_transform(&i);
            let child_bbox = self.bounding_box_per_layer(&template, layer)
                // Transform the child bounding box to ther correct position.
                .map(|b| b.transform(|p| tf.transform_point(p)));

            bbox = match (bbox, child_bbox) {
                (None, None) => None,
                (Some(b), None) | (None, Some(b)) => Some(b),
                (Some(a), Some(b)) => Some(a.add_rect(&b))
            }
        });

        bbox
    }

    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        Box::new(self.circuit(cell).shapes(layer).expect("Layer not found.")
            .each_shape().map(|s| s.index))
    }

    // fn each_shape(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=&Geometry<Self::Coord>> + '_> {
    //     Box::new(self.cells[cell].shapes_map[layer].shapes.values().map(|s| &s.geometry))
    // }

    fn for_each_shape<F>(&self, cell_id: &Self::CellId, layer: &Self::LayerId, mut f: F)
        where F: FnMut(&Self::ShapeId, &Geometry<Self::Coord>) -> () {
        self.circuits[cell_id]
            .shapes_map.get(layer)
            .into_iter()
            .for_each(|s| {
                s.shapes.values()
                    .for_each(|s| f(&s.index, &s.geometry));
            });
    }

    fn with_shape<F, R>(&self, shape_id: &Self::ShapeId, mut f: F) -> R
        where F: FnMut(&Self::LayerId, &Geometry<Self::Coord>) -> R {
        let shape = self.shape(shape_id);
        let (_, layer) = &self.shape_parents[shape_id];
        f(layer, &shape.geometry)
    }

    fn parent_of_shape(&self, shape_id: &Self::ShapeId) -> (Self::CellId, Self::LayerId) {
        self.shape_parents.get(shape_id)
            .expect("Shape ID not found.")
            .clone()
    }

    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<Self::Coord> {
        self.circuit_inst(cell_inst).get_transform().clone()
    }

    fn get_shape_property(&self, shape: &Self::ShapeId, key: &Self::NameType) -> Option<PropertyValue> {
        let (cell, layer) = self.shape_parents[shape].clone();
        self.circuit(&cell)
            .shapes_map[&layer]
            .shape_properties.get(shape)
            .and_then(|props| props.get(key))
            .cloned()
    }
}

impl HierarchyEdit for Chip<Coord> {
    fn new() -> Self {
        Chip::default()
    }

    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId {
        // TODO
        self.create_circuit(name, vec![])
    }

    fn remove_cell(&mut self, cell_id: &Self::CellId) {
        self.remove_circuit(cell_id)
    }

    fn create_cell_instance(&mut self, parent_cell: &Self::CellId, template_cell: &Self::CellId, name: Option<Self::NameType>) -> Self::CellInstId {
        let id = self.create_circuit_instance(parent_cell, template_cell, name);
        // self.circuit_inst_mut(&id).set_transform(SimpleTransform::identity());
        id
    }

    fn remove_cell_instance(&mut self, id: &Self::CellInstId) {
        <Chip<Coord>>::remove_circuit_instance(self, id)
    }


    fn rename_cell_instance(&mut self, inst: &Self::CellInstId, new_name: Option<Self::NameType>) {
        <Chip<Coord>>::rename_cell_instance(self, inst, new_name)
    }

    fn rename_cell(&mut self, cell: &Self::CellId, new_name: Self::NameType) {
        <Chip<Coord>>::rename_cell(self, cell, new_name)
    }

    fn set_chip_property(&mut self, key: Self::NameType, value: PropertyValue) {
        self.properties.insert(key, value);
    }

    fn set_cell_property(&mut self, cell: &Self::CellId, key: Self::NameType, value: PropertyValue) {
        self.circuit_mut(cell).properties.insert(key, value);
    }

    fn set_cell_instance_property(&mut self, inst: &Self::CellInstId, key: Self::NameType, value: PropertyValue) {
        self.circuit_inst_mut(inst).properties.insert(key, value);
    }
}

impl LayoutEdit for Chip<Coord> {
    fn set_dbu(&mut self, dbu: Self::Coord) {
        self.dbu = dbu;
    }

    fn create_layer(&mut self, index: u32, datatype: u32) -> Self::LayerId {
        // Find next free layer index.
        let layer_index = self.layer_index_generator.next();
        // Create new entries in the layer lookup tables.
        self.layers_by_index_datatype.insert((index, datatype), layer_index);

        let info = LayerInfo { index, datatype, name: None };
        self.layer_info.insert(layer_index, info);
        layer_index
    }

    fn set_layer_name(&mut self, layer: &Self::LayerId, name: Option<Self::NameType>) -> Option<Self::NameType> {
        if let Some(name) = &name {
            // Check that we do not shadow another layer name.
            let existing = self.layers_by_name.get(name);

            if existing == Some(layer) {
                // Nothing to be done.
                return Some(name.clone());
            }

            if existing.is_some() {
                panic!("Layer name already exists: '{}'", name)
            }
        }

        // Remove the name.
        let previous_name = self.layer_info
            .get_mut(layer)
            .expect("Layer ID not found.").name.take();
        if let Some(prev_name) = &previous_name {
            // Remove the name from the table.
            self.layers_by_name.remove(prev_name);
        }

        if let Some(name) = name {
            // Create the link from the name to the ID.
            self.layers_by_name.insert(name.clone(), *layer);
            // Store the name.
            self.layer_info
                .get_mut(layer)
                .expect("Layer ID not found.").name = Some(name);
        }
        previous_name
    }

    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId {
        let shape_id = self.shape_index_generator.next();

        let shape = Shape {
            index: shape_id,
            geometry,
            net: None,
            pin: None,
            user_data: Default::default(),
        };

        self.shape_parents.insert(shape_id.clone(), (parent_cell.clone(), layer.clone()));

        self.circuit_mut(parent_cell)
            .get_or_create_shapes_mut(layer)
            .shapes.insert(shape_id, shape);

        shape_id
    }

    fn remove_shape(&mut self, shape_id: &Self::ShapeId)
                    -> Option<Geometry<Self::Coord>> {

        // Remove all links to this shape.
        if let Some(net) = self.get_net_of_shape(shape_id) {
            self.net_mut(&net).net_shapes.remove(shape_id);
        }
        if let Some(pin) = self.get_pin_of_shape(shape_id) {
            self.pin_mut(&pin).pin_shapes.remove(shape_id);
        }

        let (parent_cell, layer) = self.shape_parents[shape_id].clone();
        self.shape_parents.remove(shape_id);

        self.circuit_mut(&parent_cell)
            .shapes_mut(&layer).expect("Layer not found.")
            .shapes.remove(shape_id)
            .map(|s| s.geometry)
    }

    fn replace_shape(&mut self, shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>)
                     -> Geometry<Self::Coord> {
        let (parent_cell, layer) = self.shape_parents[shape_id].clone();
        let shape_id = *shape_id;

        let g = &mut self.circuit_mut(&parent_cell)
            .shapes_mut(&layer).expect("Layer not found.")
            .shapes.get_mut(&shape_id).expect("Shape not found.")
            .geometry;
        let mut new_g = geometry;
        std::mem::swap(g, &mut new_g);
        new_g
    }

    fn set_transform(&mut self, cell_inst: &Self::CellInstId, tf: SimpleTransform<Self::Coord>) {
        self.circuit_inst_mut(cell_inst).set_transform(tf)
    }

    fn set_shape_property(&mut self, shape: &Self::ShapeId, key: Self::NameType, value: PropertyValue) {
        let (cell, layer) = self.shape_parents[shape].clone();
        self.circuit_mut(&cell)
            .shapes_map.get_mut(&layer).expect("Layer not found.")
            .shape_properties.entry(shape.clone())
            .or_insert(Default::default())
            .insert(key, value);
    }
}

impl L2NBase for Chip<Coord> {
    fn shapes_of_net(&self, net_id: &Self::NetId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        Box::new(self.net(net_id)
            .net_shapes.iter()
            .cloned()
        )
    }

    fn shapes_of_pin(&self, pin_id: &Self::PinId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        Box::new(self.pin(pin_id)
            .pin_shapes.iter()
            .cloned()
        )
    }

    fn get_net_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::NetId> {
        self.shape(shape_id).net.clone()
    }

    fn get_pin_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::PinId> {
        self.shape(shape_id).pin.clone()
    }
}

impl L2NEdit for Chip<Coord> {
    fn set_pin_of_shape(&mut self, shape_id: &Self::ShapeId, pin: Option<Self::PinId>) -> Option<Self::PinId> {
        // Create link from pin to shape.
        if let Some(pin) = &pin {
            let not_yet_present = self.pin_mut(pin)
                .pin_shapes.insert(shape_id.clone());
            if !not_yet_present {
                // The shape is already assigned to this pin. Don't do anything more.
                return Some(pin.clone());
            }
        }

        // Create from shape to link.
        let mut pin = pin;
        std::mem::swap(&mut self.shape_mut(shape_id).pin, &mut pin);
        let previous_pin = pin;

        // Remove the old link to the pin.
        if let Some(previous_pin) = &previous_pin {
            assert!(self.pin_mut(previous_pin)
                        .pin_shapes.remove(shape_id), "Pin was not linked to the shape.");
        }

        // Return the previous pin (got it by the above swap operation).
        previous_pin
    }

    fn set_net_of_shape(&mut self, shape_id: &Self::ShapeId, net: Option<Self::NetId>) -> Option<Self::NetId> {
        // Create link from net to shape.
        if let Some(net) = &net {
            let not_yet_present = self.net_mut(net)
                .net_shapes.insert(shape_id.clone());
            if !not_yet_present {
                // The shape is already assigned to this net. Don't do anything more.
                return Some(net.clone());
            }
        }

        // Create from shape to link.
        let mut net = net;
        std::mem::swap(&mut self.shape_mut(shape_id).net, &mut net);
        let previous_net = net;

        // Remove the old link to the net.
        if let Some(previous_net) = &previous_net {
            assert!(self.net_mut(previous_net)
                        .net_shapes.remove(shape_id), "Net was not linked to the shape.");
        }

        // Return the previous net (got it by the above swap operation).
        previous_net
    }
}