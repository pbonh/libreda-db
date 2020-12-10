
extern crate libreda_db;

use libreda_db::netlist::prelude::*;
use itertools::Itertools;


// Test circuit equality.
#[test]
fn test_circuit_eq() {
    let mut netlist = Netlist::new();
    let a = netlist.create_circuit("a", vec![]);
    let b = netlist.create_circuit("b", vec![]);
    assert_eq!(a.clone(), a.clone());
    assert_eq!(&a, &a);
    assert_ne!(a.clone(), b.clone());
    assert_ne!(&a, &b);
}

// Check if creating recursive circuits leads to an error.
#[test]
#[should_panic(expected = "Cannot create recursive instances.")]
fn test_circuit_no_recursion_1() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![]);
    // This should fail:
    let _top_inst = top.create_circuit_instance(&top, "recursive_inst");
}

#[test]
#[should_panic(expected = "Cannot create recursive instances.")]
fn test_circuit_no_recursion_2() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![]);
    let sub = netlist.create_circuit("sub", vec![]);
    let _sub_inst = top.create_circuit_instance(&sub, "sub_inst");
    // This should fail:
    let _top_inst = sub.create_circuit_instance(&top, "recursive_inst");
}

#[test]
fn test_create_and_remove_instance() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![]);
    let sub = netlist.create_circuit("sub", vec![]);
    assert_ne!(top.id(), sub.id());

    let sub_inst = top.create_circuit_instance(&sub, "sub_inst");

    assert_eq!(sub_inst.name(), Some(&"sub_inst".to_string()));

    assert_eq!(top.each_instance().collect_vec(), vec![sub_inst.clone()]);

    assert_eq!(top.num_instances(), 1);
    assert_eq!(top.num_references(), 0);
    assert_eq!(sub.num_references(), 1);

    assert_eq!(sub_inst.parent_circuit().upgrade(), Some(top.clone()));

    // Remove the instance.
    top.remove_circuit_instance(&sub_inst);
    assert_eq!(top.each_instance().collect_vec(), vec![]);

    assert_eq!(top.num_instances(), 0);
    assert_eq!(top.num_references(), 0);
    assert_eq!(sub.num_references(), 0);
}

#[test]
fn test_dependent_circuits() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![]);
    let sub = netlist.create_circuit("sub", vec![]);
    let sub_inst = top.create_circuit_instance(&sub, "sub_inst");

    assert_eq!(sub.each_dependent_circuit().collect_vec(), vec![top.clone()]);
    assert_eq!(top.each_dependent_circuit().collect_vec(), vec![]);

    assert_eq!(top.each_circuit_dependency().collect_vec(), vec![sub.clone()]);
    assert_eq!(sub.each_circuit_dependency().collect_vec(), vec![]);

    // Remove the instance and see if dependencies are updated correctly.
    top.remove_circuit_instance(&sub_inst);
    assert_eq!(sub.each_dependent_circuit().collect_vec(), vec![]);
    assert_eq!(top.each_circuit_dependency().collect_vec(), vec![]);
}

#[test]
fn test_simple_net() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![Pin::new_input("A")]);
    let a = netlist.create_circuit("a", vec![Pin::new_input("A")]);
    let b = netlist.create_circuit("b", vec![Pin::new_input("A")]);
    let a_inst = top.create_circuit_instance(&a, "a_inst");
    let b_inst = top.create_circuit_instance(&b, "b_inst");

    let net1 = top.create_net(Some("Net1"));
    assert_eq!(net1.parent_circuit().upgrade(), Some(top.clone()));

    assert_eq!(Some(net1.clone()), top.net_by_name("Net1"));

    top.connect_pin_by_id(0, net1.clone());
    a_inst.connect_pin_by_id(0, &net1);
    b_inst.connect_pin_by_id(0, &net1);

    assert_eq!(net1.num_terminals(), 3);
    assert_eq!(net1.each_terminal().count(), 3);

    assert_eq!(net1.each_terminal()
                   .filter_map(|t| match t {
                       TerminalRef::Pin(p) => Some(p),
                       _ => None
                   })
                   .count(), 1, "Number of connections to `Pin`s is wrong.");

    assert_eq!(net1.each_terminal()
                   .filter_map(|t| match t {
                       TerminalRef::PinInstance(p) => Some(p),
                       _ => None
                   })
                   .count(), 2, "Number of connections to `PinInstance`s is wrong.");

    assert_eq!(net1.each_instance().unique().count(), 2);
}


#[test]
fn test_rename_net() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![Pin::new_input("A")]);

    let net1 = top.create_net(Some("Net1"));
    assert_eq!(Some(net1.clone()), top.net_by_name("Net1"));

    // Change name.
    net1.rename(Some("NewName"));
    assert_eq!(Some(net1.clone()), top.net_by_name("NewName"));

    // Change back to original.
    net1.rename(Some("Net1"));
    assert_eq!(Some(net1.clone()), top.net_by_name("Net1"));

    // No name.
    net1.rename::<String>(None);
    assert_eq!(None, top.net_by_name("Net1"));

}

#[test]
fn test_flatten_circuit_instance() {
    let mut netlist = Netlist::new();
    let top = netlist.create_circuit("top", vec![Pin::new_input("A")]);
    let a = netlist.create_circuit("a", vec![Pin::new_input("A")]);
    let b = netlist.create_circuit("b", vec![Pin::new_input("A")]);
    let a_inst = top.create_circuit_instance(&a, "a_inst");
    let b_inst = a.create_circuit_instance(&b, "b_inst");

    let net1 = top.create_net(Some("Net1"));
    top.connect_pin_by_id(0, net1.clone());
    a_inst.connect_pin_by_id(0, &net1);

    let net2 = a.create_net(Some("Net2"));
    a.connect_pin_by_id(0, net2.clone());
    b_inst.connect_pin_by_id(0, &net2);

    // Flatten the middle circuit.
    top.flatten_circuit_instance(&a_inst);
    assert_eq!(top.num_instances(), 1);
    assert!(top.circuit_instance_by_name("a_inst").is_none());
    assert!(top.circuit_instance_by_name("b_inst").is_some());
}