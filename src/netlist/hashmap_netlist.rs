//! Alternative netlist implementaion. Not currently used, nor complete.

use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use itertools::Itertools;
use std::borrow::Borrow;
use std::hash::Hash;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CircuitId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CircuitInstId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PinId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PinInstId(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TerminalId {
    Pin(PinId),
    PinInst(PinInstId),
}

impl From<&PinId> for TerminalId {
    fn from(id: &PinId) -> Self {
        TerminalId::Pin(*id)
    }
}

impl From<&PinInstId> for TerminalId {
    fn from(id: &PinInstId) -> Self {
        TerminalId::PinInst(*id)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NetId(usize);


#[derive(Debug, Clone, Default)]
pub struct Circuit {
    /// Name of the circuit.
    pub name: String,
    /// Pin definitions.
    pub pins: Vec<PinId>,
    /// Instances inside this circuit.
    pub instances: HashSet<CircuitInstId>,
    /// Circuit instances that reference to this circuit.
    pub references: HashSet<CircuitInstId>,
    /// All circuits that have instances of this circuit.
    pub parents: HashSet<CircuitId>,
}

#[derive(Debug, Clone)]
pub struct CircuitInst {
    pub circuit: CircuitId,
    pub parent: CircuitId,
    pub pins: Vec<PinInstId>,
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub name: String,
    pub circuit: CircuitId,
    pub net: Option<NetId>,
}

#[derive(Debug, Clone)]
pub struct PinInst {
    pub pin: PinId,
    pub circuit_inst: CircuitInstId,
    pub net: Option<NetId>,
}


#[derive(Debug, Clone)]
pub struct Net {
    pub name: Rc<String>,
    pub parent: CircuitId,
    pub pins: HashSet<PinId>,
    pub pin_instances: HashSet<PinInstId>,
}

#[derive(Debug, Default)]
pub struct Netlist {
    circuits: HashMap<CircuitId, Circuit>,
    circuits_by_name: HashMap<String, CircuitId>,
    circuit_instances: HashMap<CircuitInstId, CircuitInst>,
    nets: HashMap<NetId, Net>,
    nets_by_name: HashMap<Rc<String>, NetId>,
    pins: HashMap<PinId, Pin>,
    pin_instances: HashMap<PinInstId, PinInst>,

    id_counter_circuit: usize,
    id_counter_circuit_inst: usize,
    id_counter_pin: usize,
    id_counter_pin_inst: usize,
    id_counter_net: usize,
}

impl Netlist {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn circuit_by_name<S: ?Sized + Eq + Hash>(&self, name: &S) -> Option<CircuitId>
        where String: Borrow<S> {
        self.circuits_by_name.get(name).copied()
    }

    pub fn circuit(&self, id: &CircuitId) -> &Circuit {
        &self.circuits[id]
    }

    fn circuit_mut(&mut self, id: &CircuitId) -> &mut Circuit {
        self.circuits.get_mut(id).unwrap()
    }

    pub fn circuit_inst(&self, id: &CircuitInstId) -> &CircuitInst {
        &self.circuit_instances[id]
    }

    pub fn net(&self, id: &NetId) -> &Net {
        &self.nets[id]
    }

    fn net_mut(&mut self, id: &NetId) -> &mut Net {
        self.nets.get_mut(id).unwrap()
    }

    pub fn pin(&self, id: &PinId) -> &Pin {
        &self.pins[id]
    }

    fn pin_mut(&mut self, id: &PinId) -> &mut Pin {
        self.pins.get_mut(id).unwrap()
    }

    pub fn pin_inst(&self, id: &PinInstId) -> &PinInst {
        &self.pin_instances[id]
    }

    fn pin_inst_mut(&mut self, id: &PinInstId) -> &mut PinInst {
        self.pin_instances.get_mut(id).unwrap()
    }

    fn next_id_counter(ctr: &mut usize) -> usize {
        let c = *ctr;
        *ctr += 1;
        c
    }

    fn create_pin(&mut self, parent: CircuitId, name: String) -> PinId {
        let id = PinId(Netlist::next_id_counter(&mut self.id_counter_pin));
        let pin = Pin {
            name,
            circuit: parent,
            net: Default::default(),
        };
        self.pins.insert(id, pin);
        id
    }

    fn create_pin_inst(&mut self, circuit: CircuitInstId, pin: PinId) -> PinInstId {
        let id = PinInstId(Netlist::next_id_counter(&mut self.id_counter_pin_inst));
        let pin = PinInst {
            pin: pin,
            circuit_inst: circuit,
            net: None,
        };
        self.pin_instances.insert(id, pin);
        id
    }

    pub fn create_net<S: Into<String>>(&mut self, parent: CircuitId, name: S) -> NetId {
        let name = Rc::new(name.into());
        let id = NetId(Netlist::next_id_counter(&mut self.id_counter_net));
        let net = Net {
            name: name.clone(),
            parent,
            pins: Default::default(),
            pin_instances: Default::default(),
        };
        self.nets.insert(id, net);
        self.nets_by_name.insert(name, id);
        id
    }

    pub fn create_circuit<S: Into<String>>(&mut self, name: S, pins: Vec<S>) -> CircuitId {
        let name = name.into();
        assert!(!self.circuits_by_name.contains_key(&name), "Circuit with this name already exists.");
        let id = CircuitId(Netlist::next_id_counter(&mut self.id_counter_circuit));

        // Create pins.
        let pins = pins.into_iter()
            .map(|name| self.create_pin(id, name.into()))
            .collect();

        let circuit = Circuit {
            name: name.clone(),
            pins,
            instances: Default::default(),
            references: Default::default(),
            parents: Default::default(),
        };

        self.circuits.insert(id, circuit);
        self.circuits_by_name.insert(name, id);

        id
    }

    pub fn create_circuit_instance(&mut self, parent: CircuitId, circuit_template: CircuitId) -> CircuitInstId {
        let id = CircuitInstId(Netlist::next_id_counter(&mut self.id_counter_circuit_inst));

        // Check that there is no cycle.
        // Find root.
        {
            let mut parents = vec![parent];
            loop {
                if parents.is_empty() {
                    // Found a root, there is no cycle.
                    break;
                } else {
                    if parents.contains(&circuit_template) {
                        // Loop found.
                        panic!("Cannot create loops!");
                    }
                    // Find parents on next level.
                    let new_parents = parents.iter()
                        .flat_map(|p| self.circuit(p).parents.iter().copied())
                        .collect();
                    parents = new_parents;
                }
            };
        }

        // Create pin instances from template pins.
        let pins = self.circuit(&circuit_template).pins.clone()
            .iter()
            .map(|&p| self.create_pin_inst(id, p))
            .collect();

        let inst = CircuitInst {
            circuit: circuit_template,
            parent,
            pins,
        };

        self.circuit_instances.insert(id, inst);
        self.circuit_mut(&parent).instances.insert(id);
        self.circuit_mut(&circuit_template).references.insert(id);

        id
    }

    /// Get all nets that are connected to the circuit instance.
    pub fn circuit_inst_nets(&self, circuit_inst_id: &CircuitInstId) -> impl Iterator<Item=NetId> + '_ {
        self.circuit_inst(circuit_inst_id).pins.iter()
            .flat_map(move |p| self.pin_inst(p).net)
    }

    pub fn each_pin_instance(&self, circuit_inst_id: &CircuitInstId) -> impl Iterator<Item=PinInstId> + '_ {
        self.circuit_inst(circuit_inst_id).pins.iter().copied()
    }

    /// Iterate over all pins of a circuit.
    pub fn each_pin(&self, circuit_id: &CircuitId) -> impl Iterator<Item=PinId> + '_ {
        self.circuit(circuit_id).pins.iter().copied()
    }

    pub fn pins_for_net(&self, net: &NetId) -> impl Iterator<Item=PinId> + '_ {
        self.net(net).pins.iter().copied()
    }

    pub fn pins_instances_for_net(&self, net: &NetId) -> impl Iterator<Item=PinInstId> + '_ {
        self.net(net).pin_instances.iter().copied()
    }

    pub fn terminals_for_net(&self, net: &NetId) -> impl Iterator<Item=TerminalId> + '_ {
        self.pins_for_net(net).map(|p| TerminalId::Pin(p))
            .chain(self.pins_instances_for_net(net).map(|p| TerminalId::PinInst(p)))
    }

    /// Disconnect all connected terminals and remove the net.
    pub fn remove_net(&mut self, net: &NetId) {
        let pins = self.pins_for_net(net).collect_vec();
        let pin_insts = self.pins_instances_for_net(net).collect_vec();

        for p in pins {
            self.disconnect_pin(&p);
        }
        for p in pin_insts {
            self.disconnect_pin(&p);
        }
        let name = self.net(&net).name.clone();
        self.nets.remove(&net).unwrap();
        self.nets_by_name.remove(&name).unwrap();
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

    pub fn remove_circuit_instance(&mut self, circuit_inst_id: &CircuitInstId) {
        // Disconnect all pins first.
        for pin in self.circuit_inst(circuit_inst_id).pins.clone() {
            self.disconnect_pin(&pin);
        }
        // Remove the instance and all references.
        let parent = self.circuit_inst(&circuit_inst_id).parent;
        let template = self.circuit_inst(&circuit_inst_id).circuit;
        self.circuit_instances.remove(&circuit_inst_id).unwrap();
        self.circuit_mut(&parent).instances.remove(circuit_inst_id);
        self.circuit_mut(&template).references.remove(circuit_inst_id);
    }

    /// Remove all instances inside the circuit,
    pub fn remove_circuit(&mut self, circuit_id: &CircuitId) {
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

    pub fn connect_pin<T: Into<TerminalId>>(&mut self, pin: T, net: Option<NetId>) {
        let t = pin.into();

        match t {
            TerminalId::Pin(p) => {
                if let Some(net) = net {
                    assert_eq!(self.pin(&p).circuit, self.net(&net).parent, "Pin and net do not live in the same circuit.");
                    self.net_mut(&net).pins.insert(p);
                }
                self.pin_mut(&p).net = net;
            }
            TerminalId::PinInst(p) => {
                if let Some(net) = net {
                    assert_eq!(self.circuit_inst(&self.pin_inst(&p).circuit_inst).parent,
                               self.net(&net).parent, "Pin and net do not live in the same circuit.");
                    self.net_mut(&net).pin_instances.insert(p);
                }
                self.pin_inst_mut(&p).net = net;
            }
        }
    }

    pub fn disconnect_pin<T: Into<TerminalId>>(&mut self, pin: T) {
        self.connect_pin(pin, None);
    }

    /// Return number of top level circuits (roots of the circuit tree).
    pub fn top_circuit_count(&self) -> usize {
        self.circuits.values()
            .filter(|c| c.parents.len() == 0)
            .count()
    }

    pub fn each_circuit(&self) -> impl Iterator<Item=CircuitId> + '_ {
        self.circuits.keys().copied()
    }

    /// Iterate through all nets that are defined in the netlist.
    pub fn each_net(&self) -> impl Iterator<Item=NetId> + '_ {
        self.nets.keys().copied()
    }
}

#[test]
fn test_create_populated_netlist() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("TOP", vec!["A", "B"]);
    assert_eq!(Some(top), netlist.circuit_by_name("TOP"));

    let sub_a = netlist.create_circuit("SUB_A", vec!["A", "B"]);
    let sub_b = netlist.create_circuit("SUB_B", vec!["A", "B"]);

    let inst_a = netlist.create_circuit_instance(top, sub_a);
    let _inst_b = netlist.create_circuit_instance(top, sub_b);

    let net_a = netlist.create_net(top, "NetA");
    let net_b = netlist.create_net(top, "NetB");

    let pins_a = netlist.each_pin_instance(&inst_a).collect_vec();
    let pins_top = netlist.each_pin(&top).collect_vec();

    netlist.connect_pin(&pins_a[0], Some(net_a));
    netlist.connect_pin(&pins_a[1], Some(net_b));

    netlist.connect_pin(&pins_top[0], Some(net_a));
    netlist.connect_pin(&pins_top[1], Some(net_b));

    dbg!(&netlist);
    dbg!(netlist.terminals_for_net(&net_a).collect_vec());
}