/*
 * Copyright (c) 2020-2020 Thomas Kramer.
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
//! A `Circuit` is a template for circuit instances. It is defined by pins interfacing to the outside
//! of the circuit, sub-circuits that live inside the circuit and nets that do the internal connections.

use super::prelude::*;
use std::cell::RefCell;
use std::rc::{Weak, Rc};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use genawaiter::rc::Gen;
use itertools::Itertools;
use std::ops::Deref;
use std::borrow::Borrow;
use std::collections::hash_map::Values;
use std::fmt;

/// Copied from KLayout: Circuits are the basic building blocks of the netlist.
/// A circuit has pins by which it can connect to the outside. Pins are created using create_pin and are represented by the Pin class.
pub struct Circuit {
    /// Name of the circuit.
    pub(super) name: String,
    /// Index of the circuit. This is automatically set when creating a circuit in the `Netlist`.
    /// The ID uniquely identifies a circuit within the netlist.
    id: CircuitIndex,
    /// Reference to this circuit itself.
    pub(super) self_reference: RefCell<Weak<Circuit>>,
    /// Pins of the circuit towards the outside.
    pins: Vec<Rc<Pin>>,
    /// The nets that are defined inside this circuit.
    nets: RefCell<HashMap<NetIndex, Rc<Net>>>,
    /// Nets indexed by name.
    nets_by_name: RefCell<HashMap<String, NetIndex>>,
    /// Generator for creating net IDs.
    /// Starts at 2 because 0 and 1 are reserved for constant LOW and HIGH nets.
    net_index_generator: RefCell<NetIndexGenerator>,
    /// Sub-circuit instances.
    circuit_instances: RefCell<HashMap<CircuitInstIndex, Rc<CircuitInstance>>>,
    /// Sub-circuit instances indexed by name.
    circuit_instances_by_name: RefCell<HashMap<String, CircuitInstIndex>>,
    /// Generator for creating sub-circuit instance IDs.
    circuit_instance_index_generator: RefCell<CircuitInstIndexGenerator>,
    /// All the instances of this circuit.
    circuit_references: RefCell<HashSet<Rc<CircuitInstance>>>,
    /// Set of circuits that are dependencies of this circuit.
    /// Stored together with a weak reference and a counter of how many instances of the dependency are present.
    /// This are the circuits towards the leaves in the dependency tree.
    dependencies: RefCell<HashMap<CircuitIndex, (Weak<Circuit>, usize)>>,
    /// Circuits that use a instance of this circuit.
    /// This are the circuits towards the root in the dependency tree.
    dependent_circuits: RefCell<HashMap<CircuitIndex, (Weak<Circuit>, usize)>>,
}

impl fmt::Debug for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Circuit")
            .field("name", &self.name)
            .field("id", &self.id.value())
            .field("pins", &self.pins.iter().map(|p| p.name()).collect_vec())
            // .field("nets", &self.nets)
            .finish()
    }
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut circuit_instances = self.each_instance().collect_vec();
        circuit_instances.sort_by_key(|c| c.id());

        // Get pin names together with the net they are connected to.
        let pin_names = self.each_pin()
            .map(|p| {
                let netname = p.internal_net().map(|n| n.create_name());
                format!("{}={:?}", p.name(), netname)
            }).join(" ");

        writeln!(f, ".subckt {} {}", self.name, pin_names)?;
        for c in circuit_instances {
            // fmt::Debug::fmt(Rc::deref(&c), f)?;
            let sub_name = c.name().cloned()
                .unwrap_or_else(|| format!("__{}", c.id));
            let sub_template = c.circuit_ref().upgrade().unwrap()
                .name().clone();
            let nets = c.each_pin_instance()
                .map(|p| {
                    let netname = p.net().map(|n| n.create_name());
                    format!("{}={:?}", p.pin().name(), netname)
                }).join(" ");
            writeln!(f, "    X{} {} {}", sub_name, sub_template, nets)?;
        }
        writeln!(f, ".ends {}\n", self.name)?;

        fmt::Result::Ok(())
    }
}

impl Eq for Circuit {}

impl PartialEq for Circuit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
        // TODO: Compare parent netlists somehow.
    }
}

impl Hash for Circuit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Circuit {
    /// Create a new circuit.
    ///
    /// This is not exposed to the crate API. Instead, circuits shall be created
    /// with `Netlist::create_circuit()`.
    pub(super) fn new(circuit_id: CircuitIndex, name: String, pins: Vec<Pin>) -> Self {

        // Set the pin IDs and put the pins into an `Rc`.
        let pins = pins.into_iter().enumerate()
            .map(|(pin_id, mut p)| {
                p.id = pin_id;
                p.parent_circuit_id = circuit_id;
                Rc::new(p)
            })
            .collect();

        Circuit {
            name,
            id: circuit_id,
            self_reference: Default::default(),
            pins,
            nets: Default::default(),
            nets_by_name: Default::default(),
            net_index_generator: RefCell::new(NetIndexGenerator::new(2)), // 0 and 1 are special nets.
            circuit_instances: Default::default(),
            circuit_instances_by_name: Default::default(),
            circuit_instance_index_generator: Default::default(),
            circuit_references: Default::default(),
            dependencies: Default::default(),
            dependent_circuits: Default::default(),
        }
    }

    /// Get the name of this circuit.
    pub fn name(&self) -> &String {
        &self.name
    }


    /// Connects pin with the given internal net.
    ///
    /// Returns the previously connected net.
    ///
    /// # Panics
    /// Panics if the pin with this ID does not exist.
    pub fn connect_pin_by_id<N: Into<Option<Rc<Net>>>>(&self, pin_id: usize, net: N) -> Option<Rc<Net>> {
        // Find the pin.
        let pin = self.pins.get(pin_id).expect("Pin with this index does not exist.");
        self.connect_pin(&pin, net)
    }

    /// Connects pin with the given internal net.
    ///
    /// Returns the previously connected net.
    ///
    /// # Panics
    /// Panics if the pin does not live in this circuit.
    pub fn connect_pin<N: Into<Option<Rc<Net>>>>(&self, pin: &Rc<Pin>, net: N) -> Option<Rc<Net>> {
        // Check that this pin actually lives in this circuit.
        assert_eq!(pin.parent_circuit_id, self.id, "Pin does not live in this circuit.");
        debug_assert!(pin.parent_circuit().ptr_eq(&self.self_reference()));

        let net = net.into();

        // Check that the net lives in this circuit.
        if let Some(net) = &net {
            assert!(net.parent_circuit.ptr_eq(&self.self_reference()),
                    "Net does not live in this circuit.");
        }

        let old_net = pin.internal_net();

        if let Some(old_net) = &old_net {
            // Remove this terminal from the old net.
            old_net.pins.borrow_mut().remove(&pin.clone());
        }

        if let Some(net) = &net {
            // Add the terminal to the net.
            net.pins.borrow_mut()
                .insert(pin.clone());
        }

        // Write the net into the pin instance.
        pin.internal_net.replace(net);

        old_net
    }

    /// Get the net of the logical constant zero or one.
    fn net_of_logic_constant(&self, constant: usize) -> Rc<Net> {
        let net_index = NetIndex::new(constant);
        self.net_by_index(&net_index)
            .unwrap_or_else(|| {
                // Create the new net.
                let net = Rc::new(
                    Net {
                        id: net_index,
                        name: Default::default(),
                        parent_circuit: self.self_reference.borrow().clone(),
                        pins: Default::default(),
                        pin_instances: Default::default(),
                    }
                );

                // Store the net.
                self.nets.borrow_mut().insert(net_index, net.clone());

                net
            })
    }

    /// Get the net of the logical constant zero.
    pub fn net_zero(&self) -> Rc<Net> {
        self.net_of_logic_constant(0)
    }

    /// Get the net of the logical constant one.
    pub fn net_one(&self) -> Rc<Net> {
        self.net_of_logic_constant(1)
    }

    /// Create a named or anonymous net.
    pub fn create_net<S: Into<String>>(&self, name: Option<S>) -> Rc<Net> {
        let name = name.map(|n| n.into());

        // Create a new ID.
        let net_index = self.net_index_generator.borrow_mut().next();

        // Check if the name is not already used and create a link between name and ID.
        if let Some(name) = &name {
            assert!(!self.nets_by_name.borrow().contains_key(name), "Net name already exists.");
            self.nets_by_name.borrow_mut().insert(name.clone(), net_index);
        }

        // Create the new net.
        let net = Rc::new(
            Net {
                id: net_index,
                name: RefCell::new(name),
                parent_circuit: self.self_reference.borrow().clone(),
                pins: Default::default(),
                pin_instances: Default::default(),
            }
        );

        // Store the net.
        self.nets.borrow_mut().insert(net_index, net.clone());

        net
    }

    /// Get all circuits (not instances) that are direct children of this circuit.
    pub fn each_circuit_dependency(&self) -> impl Iterator<Item=Rc<Circuit>> + '_ {
        let generator = Gen::new(|co| async move {
            for (dep, _counter) in self.dependencies.borrow().values() {
                co.yield_(dep.upgrade().unwrap()).await;
            }
        });
        generator.into_iter()
    }

    /// Get all circuits that directly depend on this circuit,
    /// i.e. have an instance of this circuit as a direct child.
    pub fn each_dependent_circuit(&self) -> impl Iterator<Item=Rc<Circuit>> + '_ {
        let generator = Gen::new(|co| async move {
            for (dep, _counter) in self.dependent_circuits.borrow().values() {
                co.yield_(dep.upgrade().unwrap()).await;
            }
        });
        generator.into_iter()
    }

    /// Create a new named instance of a given circuit.
    /// # Panics
    /// Panics if the instantiation is recursive.
    pub fn create_circuit_instance<S: Into<String>>(&self, template_circuit: &Rc<Circuit>, name: S) -> Rc<CircuitInstance> {
        let name = name.into();
        // Check if name already exists.
        if self.circuit_instances_by_name.borrow().contains_key(&name) {
            panic!(format!("Instance with name '{}' already exists.", &name));
        }

        {
            // Check that creating this circuit instance does not create a cycle in the dependency graph.
            // There can be no recursive instances.
            let mut stack: Vec<Rc<Circuit>> = vec![self.self_reference().upgrade().unwrap()];
            while let Some(c) = stack.pop() {
                if c.eq(template_circuit) {
                    // The circuit to be instantiated depends on the current circuit.
                    // This would insert a loop into the dependency tree.
                    // TODO: Don't panic but return an `Err`.
                    panic!("Cannot create recursive instances.");
                }
                // Follow the dependent circuits towards the root.
                c.dependent_circuits.borrow().values()
                    .map(|(dep, _)| dep.upgrade().unwrap()) // By construction this references should always be defined.
                    .for_each(|dep| stack.push(dep));
            }
        }

        let index = self.circuit_instance_index_generator.borrow_mut().next();

        // Create pin instances.
        let pin_instances: Vec<_> = template_circuit.pins.iter()
            .map(|pin|
                Rc::new(PinInstance {
                    circuit_instance_id: index,
                    circuit_instance: Default::default(),
                    pin: pin.clone(),
                    net: Default::default(),
                })
            ).collect();

        let inst = CircuitInstance {
            name: Some(name.clone()),
            circuit: Rc::downgrade(template_circuit),
            circuit_id: template_circuit.id,
            parent_circuit: self.self_reference(),
            parent_circuit_id: self.id,
            id: index,
            pin_instances: pin_instances.clone(),
        };
        let inst = Rc::new(inst);

        // Store weak reference to circuit instance into the pin instances.
        for pi in pin_instances {
            pi.circuit_instance.replace(Rc::downgrade(&inst));
        }

        // Create entry in name lookup table.
        self.circuit_instances_by_name.borrow_mut()
            .insert(name, index);

        // Store circuit instance.
        self.circuit_instances.borrow_mut()
            .insert(index, inst.clone());

        // Remember dependency.
        {
            let mut dependencies = self.dependencies.borrow_mut();
            dependencies.entry(template_circuit.id)
                .and_modify(|(_, c)| *c += 1)
                .or_insert((Rc::downgrade(template_circuit), 1)); // First entry: Save weak reference with counter = 1.
        }

        // Remember dependency.
        {
            let mut dependent = template_circuit.dependent_circuits.borrow_mut();
            dependent.entry(self.id)
                .and_modify(|(_, c)| *c += 1)
                .or_insert((self.self_reference(), 1));// First entry: Save weak reference with counter = 1.
        }

        // Create an entry in the template circuit.
        let was_not_present = template_circuit.circuit_references.borrow_mut()
            .insert(inst.clone());
        debug_assert!(was_not_present, "Circuit instance with this index already existed!");

        // Sanity checks.
        #[cfg(debug_assertions)] {
            debug_assert_eq!(self.num_references(), self.dependent_circuits.borrow().values()
                .map(|(_, n)| n).sum(), "self.num_references() is not consistent with the number of dependent circuits.");
            debug_assert_eq!(template_circuit.num_references(), template_circuit.dependent_circuits.borrow().values()
                .map(|(_, n)| n).sum(), "circuit.num_references() is not consistent with the number of dependent circuits.");

            // Check that dependencies are consistent.
            let dependencies = self.dependencies.borrow()
                .values().map(|(c, _)| c.upgrade().unwrap().id()).sorted().collect_vec();

            let dependencies_derived = self.each_instance()
                .map(|c| c.circuit_id())
                .unique().sorted().collect_vec();

            debug_assert_eq!(dependencies, dependencies_derived);
        }

        // Return reference to circuit instance.
        inst
    }

    /// Disconnects the pin from the internal net.
    ///
    /// # Panics
    /// Panics if the pin with this ID does not exist.
    pub fn disconnect_pin_by_id(&self, pin_id: usize) -> () {
        self.connect_pin_by_id(pin_id, None);
    }

    /// Disconnects the pin from the internal net.
    ///
    /// # Panics
    /// Panics if the pin does not live in this circuit.
    pub fn disconnect_pin(&self, pin: &Rc<Pin>) -> () {
        self.connect_pin(pin, None);
    }

    /// Call a closure for each child recursively.
    pub fn for_each_child_recursive<F>(&self, f: F)
        where F: Fn(&Rc<CircuitInstance>) {
        for i in self.circuit_instances.borrow().values() {
            f(i);

            let circuit = i.circuit.upgrade().unwrap();
            circuit.for_each_child_recursive(&f)
        }
    }

    /// Iterate recursively over all child circuit instances.
    /// TODO: This does not work as intended yet. Create test cases.
    pub fn each_child_recursive(&self) -> impl Iterator<Item=Rc<CircuitInstance>> + '_ {
        let generator = Gen::new(|co| async move {
            let mut visited = HashSet::new();
            let mut stack: Vec<_> = self.circuit_instances.borrow().values().cloned().collect();

            while let Some(top) = stack.pop() {
                co.yield_(top.clone()).await;

                dbg!(stack.len());
                visited.insert(top.clone());

                let circuit: Rc<Circuit> = top.circuit.upgrade().unwrap();

                for inst in circuit.circuit_instances.borrow().values().cloned() {
                    dbg!(&inst.name);
                    stack.push(inst)
                }
            }
        });
        generator.into_iter()
    }

    /// Get the number of circuit instances that live in this circuit.
    pub fn num_instances(&self) -> usize {
        self.circuit_instances.borrow().len()
    }

    /// Borrow a reference to the instance hash map.
    pub fn instances(&self) -> impl Deref<Target=HashMap<CircuitInstIndex, Rc<CircuitInstance>>> + '_ {
        self.circuit_instances.borrow()
    }

    /// Iterate over all sub circuit instances that live in this circuit.
    /// Hint: `with_instance_iter()` might be more performant.
    pub fn each_instance(&self) -> impl Iterator<Item=Rc<CircuitInstance>> + '_ {
        let generator = Gen::new(|co| async move {
            for e in self.circuit_instances.borrow().values().cloned() {
                co.yield_(e).await;
            }
        });
        generator.into_iter()
    }

    /// Iterate over all instances.
    pub fn with_instance_iter<F, R>(&self, f: F) -> R
        where F: FnOnce(Values<CircuitInstIndex, Rc<CircuitInstance>>) -> R,
    {
        f(self.circuit_instances.borrow().values())
    }

    /// Borrow a reference to the reference hash map.
    pub fn references(&self) -> impl Deref<Target=HashSet<Rc<CircuitInstance>>> + '_ {
        self.circuit_references.borrow()
    }

    /// Get the number of circuit instances that reference this circuit.
    pub fn num_references(&self) -> usize {
        self.circuit_references.borrow().len()
    }

    /// Test if the circuit has references.
    pub fn has_references(&self) -> bool {
        !self.circuit_references.borrow().is_empty()
    }

    /// Iterate over all circuit instances that reference this circuit.
    pub fn each_reference(&self) -> impl Iterator<Item=Rc<CircuitInstance>> + '_ {
        let generator = Gen::new(|co| async move {
            for e in self.circuit_references.borrow().iter().cloned() {
                co.yield_(e).await;
            }
        });
        generator.into_iter()
    }

    /// Iterate over all circuit instances that reference this circuit.
    pub fn with_reference_iter<F, R>(&self, f: F) -> R
        where F: FnOnce(std::collections::hash_set::Iter<Rc<CircuitInstance>>) -> R,
    {
        f(self.circuit_references.borrow().iter())
    }

    /// Borrow a reference to the net hash map.
    #[inline]
    pub fn nets(&self) -> impl Deref<Target=HashMap<NetIndex, Rc<Net>>> + '_ {
        self.nets.borrow()
    }

    /// Get the number of internal nets in this cell.
    pub fn num_nets(&self) -> usize {
        self.nets().len()
    }

    /// Iterate over all internal nets of this cell.
    /// Hint: `with_net_iter()` might be more performant.
    pub fn each_net(&self) -> impl Iterator<Item=Rc<Net>> + '_ {
        // Using a generator makes it possible to return an iterator over a value
        // borrowed from a `RefCell`.
        let generator = Gen::new(|co| async move {
            for n in self.nets().values().cloned() {
                co.yield_(n).await;
            }
        });
        generator.into_iter()
    }

    /// Iterate over all internal nets of this cell.
    pub fn with_net_iter<F, R>(&self, f: F) -> R
        where F: FnOnce(Values<NetIndex, Rc<Net>>) -> R,
    {
        f(self.nets().values())
    }

    /// Iterate over all pins.
    pub fn each_pin(&self) -> impl Iterator<Item=&Rc<Pin>> + ExactSizeIterator {
        self.pins.iter()
    }

    /// Get a `Vec` with all pins.
    pub fn each_pin_vec(&self) -> Vec<Rc<Pin>> {
        self.pins.to_vec()
    }

    /// Replace the circuit instance with its contents. Remove the circuit instance afterwards.
    /// Does not purge nets nor unconnected instances.
    /// So there could be unconnected nets or unconnected instances.
    ///
    /// Nets keep their names if possible. If the net name already exists in this circuit, the name will
    /// be set to `None`.
    ///
    /// The content of the circuit instance will be renamed by appending the names like a path.
    pub fn flatten_circuit_instance(&self, circuit_instance: &Rc<CircuitInstance>) {
        assert!(self.contains_instance(circuit_instance),
                "Instance does not live in this circuit.");

        // Get the template circuit.
        let template = circuit_instance.circuit_ref().upgrade().unwrap();

        // Mapping from old to new nets.
        let mut net_mapping: HashMap<Rc<Net>, Rc<Net>> = HashMap::new();

        // Get or create a new net as an equivalent of the old.
        let mut get_new_net = |old_net: Rc<Net>| -> Rc<Net> {
            if let Some(net_net) = net_mapping.get(&old_net) {
                net_net.clone()
            } else {
                // Get the name of the net.
                let net_name = old_net.name();
                // Resolve net name collisions.
                // It is possible that the net name already exists in this circuit.
                let net_name = if let Some(net_name) = net_name {
                    // Check if net name already exists.
                    if let Some(_) = self.net_index_by_name(&net_name) {
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
                let new_net = self.create_net(net_name);
                // Remember the mapping old_net -> net_net.
                net_mapping.insert(old_net.clone(), new_net.clone());
                new_net
            }
        };

        // Name of the instance to be flattened.
        let inst_name = circuit_instance.name().unwrap();

        // Copy all sub instances into this circuit.
        // And connect their pins to copies of the original nets.
        for sub in template.each_instance() {
            let sub_template = sub.circuit_ref().upgrade().unwrap();

            // TODO: Avoid instance name collisions.
            // It is possible that the instance name already exists in this circuit.

            let sub_instance_name = sub.name().unwrap();
            // Construct name for the new sub instance.
            // Something like: INSTANCE_TO_BE_FLATTENED:SUB_INSTANCE{_COUNTER}
            let new_name = {
                let mut new_name = format!("{}:{}", inst_name, sub_instance_name);
                let mut i = 0;
                // If this name too already exists, append a counter.
                while self.circuit_instances_by_name.borrow().contains_key(&new_name) {
                    new_name = format!("{}:{}_{}", inst_name, sub_instance_name, i);
                    i += 1;
                }
                new_name
            };
            let new_inst = self.create_circuit_instance(&sub_template,
                                                        new_name);

            // Re-connect pins to copies of the original nets.
            // Loop over old/new pin instances.
            for (old_pin, new_pin)
            in sub.each_pin_instance()
                .zip(new_inst.each_pin_instance()) {
                // Get net on old pin.
                if let Some(old_net) = old_pin.net() {
                    let new_net = get_new_net(old_net);
                    new_pin.connect_net(&new_net);
                }
            }
        }

        // Connect the newly created sub-instances and nets to this circuit
        // according to the old connections to the instance which is about to be flattened.
        {
            // First create a the mapping from inner nets to outer nets.
            // This is necessary for the case when multiple internal pins are connected to the same
            // internal net.
            let net_replacement_mapping: HashMap<_, _> = circuit_instance.each_pin_instance()
                .filter_map(|old_pin| {
                    let outer_old_net = old_pin.net();
                    let inner_old_net = old_pin.pin().internal_net();
                    // If the pin was either not connected on the outside or not connected inside, nothing
                    // needs to be done.
                    if let (Some(outer_net), Some(inner_old_net)) = (outer_old_net, inner_old_net) {
                        // Get the new copy of the inner net.
                        let inner_new_net = get_new_net(inner_old_net);
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

    /// Get the ID of this circuit.
    /// The ID uniquely identifies a circuit within the netlist.
    pub fn id(&self) -> CircuitIndex {
        self.id
    }

    /// Check if the instance is a child of this cell.
    pub fn contains_instance(&self, circuit_instance: &Rc<CircuitInstance>) -> bool {
        if let Some(parent) = circuit_instance.parent_circuit().upgrade() {
            self.eq(&parent)
        } else {
            false
        }
    }

    /// Return the number of defined nets in this circuit.
    /// This also includes unconnected nets.
    pub fn net_count(&self) -> usize {
        self.nets().len()
    }

    /// Get a net by its index.
    /// Returns `None` if there is no net with this index.
    pub fn net_by_index(&self, index: &NetIndex) -> Option<Rc<Net>> {
        self.nets().get(index).cloned()
    }

    /// Get a net by its name.
    /// Returns `None` if there is no net with this name.
    pub fn net_index_by_name<S: ?Sized>(&self, name: &S) -> Option<NetIndex>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.nets_by_name.borrow().get(name).cloned()
    }

    /// Find net by its name. Returns `None` if the net name does not exist.
    pub fn net_by_name<S: ?Sized>(&self, name: &S) -> Option<Rc<Net>>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.net_index_by_name(name)
            .map(|i| self.net_by_index(&i).unwrap())
    }

    /// Get the net connected to this pin.
    ///
    /// # Panics
    /// Panics if the pin does not exist.
    pub fn net_for_pin(&self, pin_id: usize) -> Option<Rc<Net>> {
        self.pin_by_id(pin_id)
            .expect("Pin does not exist.")
            .internal_net()
    }

    /// Get a pin by its ID.
    /// Returns `None` if the ID does not exist.
    pub fn pin_by_id(&self, pin_id: usize) -> Option<Rc<Pin>> {
        self.pins.get(pin_id).cloned()
    }

    /// Find a pin by its name.
    /// Returns `None` if the name is not found.
    pub fn pin_by_name(&self, pin_name: &str) -> Option<Rc<Pin>> {
        // Find the pin name by linear search.
        // TODO: Create look-up table for pin names in `Circuit`?
        let pin = self.pins.iter()
            .find(|p| p.name() == pin_name);

        pin.cloned()
    }

    /// Get the number of pins.
    pub fn pin_count(&self) -> usize {
        self.pins.len()
    }

    /// Remove all floating nets (nets that are not connected to any pin).
    /// Returns the number of purged nets.
    pub fn purge_nets(&self) -> usize {
        // Find nets that are not connected to anything.
        let unused_nets: Vec<_> = self.each_net().into_iter()
            .filter(|net| net.num_terminals() == 0).collect();

        for net in &unused_nets {
            self.remove_net(net)
        }

        unused_nets.len()
    }

    /// Remove the given net from this circuit and disconnect every pin connected to this net.
    pub fn remove_net(&self, net: &Rc<Net>) -> () {
        assert!(self.nets.borrow().contains_key(&net.id), "Net does not exist in this circuit.");
        // Disconnect all pins that are attached to this net.
        for terminal in net.each_terminal().collect_vec() {
            match terminal {
                TerminalRef::PinInstance(p) => {
                    let circuit_instance = self.circuit_instance_by_id(&p.circuit_instance_id).unwrap();
                    // Check that the pin instance belongs to a circuit instance that lives in this circuit.
                    debug_assert!(circuit_instance.parent_circuit.ptr_eq(&self.self_reference()));
                    // Disconnect the pin.
                    circuit_instance.disconnect_pin_by_id(p.id());
                }
                TerminalRef::Pin(p) => {
                    assert_eq!(p.parent_circuit_id, self.id);
                    debug_assert!(p.parent_circuit().ptr_eq(&self.self_reference()));
                    self.disconnect_pin_by_id(p.id);
                }
            }
        }

        assert_eq!(net.num_terminals(), 0, "Net must be floating now.");

        // Remove the net.
        self.nets.borrow_mut().remove(&net.id).unwrap();
    }

    /// Change the name of the net.
    ///
    /// # Panics
    /// * Panics if there is already a net with this name.
    /// * Panics if the net does not live in this circuit.
    pub(super) fn rename_net<S: Into<String>>(&self, net_index: NetIndex, name: Option<S>) {
        let net = self.net_by_index(&net_index).expect("Net not found.");

        let name: Option<String> = name.map(|n| n.into());
        // Check if a net with this name already exists.
        if let Some(name) = &name {
            if let Some(other) = self.net_by_name(name) {
                if other != net {
                    panic!("Net name already exists.")
                } else {
                    return;
                }
            }
        }

        let maybe_old_name = net.name.replace(name.clone());
        let mut nets_by_name = self.nets_by_name.borrow_mut();
        // Remove the old name mapping.
        if let Some(old_name) = maybe_old_name {
            nets_by_name.remove(&old_name);
        }
        // Add the new name mapping.
        if let Some(name) = name {
            nets_by_name.insert(name, net.id());
        }
    }

    /// Take all terminals that are connected to `old_net` and connect them to `new_net` instead.
    pub fn replace_net(&self, old_net: &Rc<Net>, new_net: &Rc<Net>) {
        // Check that the nets live in this circuit.
        assert!(old_net.parent_circuit().ptr_eq(&self.self_reference()));
        assert!(new_net.parent_circuit().ptr_eq(&self.self_reference()));
        assert!(self.nets.borrow().contains_key(&old_net.id), "Old net does not exist in this circuit.");
        assert!(self.nets.borrow().contains_key(&new_net.id), "New net does not exist in this circuit.");
        // Get terminals connected to the old net.
        let terminals = old_net.each_terminal().collect_vec();
        // Connect each terminal to the new net.
        for terminal in terminals {
            match terminal {
                TerminalRef::Pin(p) => { p.connect_net(new_net.clone()); }
                TerminalRef::PinInstance(p) => { p.connect_net(&new_net.clone()); }
            }
        }
        // Remove the now unused old net.
        self.remove_net(old_net);
    }

    /// Remove the given sub circuit instance from this circuit.
    /// # Panics
    /// Panics if the circuit instance does not live in this circuit.
    /// TODO: Return an Err and let the user decide how to handle the error.
    pub fn remove_circuit_instance(&self, circuit_instance: &Rc<CircuitInstance>) -> () {
        assert!(circuit_instance.parent_circuit().ptr_eq(&self.self_reference()),
                "Circuit instance does not live in this circuit.");

        // Remove dependency.
        {
            let mut dependencies = self.dependencies.borrow_mut();

            // Decrement counter.
            let (_, count) = dependencies.entry(circuit_instance.circuit_id())
                .or_insert((Weak::new(), 0));
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                dependencies.remove(&circuit_instance.circuit_id());
            }
        }

        // Remove dependency.
        {
            let circuit_ref = circuit_instance.circuit_ref().upgrade().unwrap();

            let mut dependent = circuit_ref.dependent_circuits.borrow_mut();

            // Decrement counter.
            let (_, count) = dependent.entry(self.id())
                .or_insert((Weak::new(), 0));
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                dependent.remove(&self.id());
            }
        }

        // Disconnect all pins of the subcircuit instance.
        for i in 0..circuit_instance.pin_count() {
            circuit_instance.disconnect_pin_by_id(i);
        }

        // Remove the subcircuit.
        for name in circuit_instance.name() {
            self.circuit_instances_by_name.borrow_mut().remove(name.as_str());
        }
        // Remove the subcircuit.
        self.circuit_instances.borrow_mut().remove(&circuit_instance.id())
            .unwrap();

        // Remove entry in the template circuit.
        let remove_successful = circuit_instance.circuit_ref().upgrade().unwrap()
            .circuit_references.borrow_mut()
            .remove(circuit_instance);
        assert!(remove_successful, "Failed to remove circuit instance from 'circuit_references'.");

        // Sanity checks.
        #[cfg(debug_assertions)]
            {
                debug_assert_eq!(self.num_references(), self.dependent_circuits.borrow().values()
                    .map(|(_, n)| n).sum());
                let instance_ref = circuit_instance.circuit_ref().upgrade().unwrap();
                debug_assert_eq!(instance_ref.num_references(), instance_ref.dependent_circuits.borrow().values()
                    .map(|(_, n)| n).sum());
            }
    }

    /// Get weak reference to this circuit.
    fn self_reference(&self) -> Weak<Self> {
        self.self_reference.borrow().clone()
    }

    /// Gets a reference to the sub circuit with the given index.
    /// Returns `None` if there is no sub circuit with this index.
    pub fn circuit_instance_by_id(&self, id: &CircuitInstIndex) -> Option<Rc<CircuitInstance>> {
        self.circuit_instances.borrow().get(id)
            .cloned()
    }

    /// Gets a reference to the sub circuit with the given name.
    /// Returns `None` if there is no sub circuit with this name.
    pub fn circuit_instance_by_name<S: ?Sized>(&self, name: &S) -> Option<Rc<CircuitInstance>>
        where String: Borrow<S>,
              S: Hash + Eq {
        self.circuit_instances_by_name.borrow().get(name)
            .and_then(|i| self.circuit_instance_by_id(i))
    }
}