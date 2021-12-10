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

use crate::traits::HierarchyBase;
use crate::prelude::PropertyValue;
use std::marker::PhantomData;
use std::ops::Deref;


// macro_rules! delegate_base {
//  ($base:tt, $B:tt) => {
//     fn test1(&self) -> i32 {
//         self.$base().test1()
//     }
//     // fn test2(&self) -> i32 {
//     //     self.$base().test2()
//     // }
//     // fn test_ref(&self) -> &i32 {
//     //     self.$base().test_ref()
//     // }
//     fn test_ref<'a>(&'a self) -> &'a i32
//         where B: 'a {
//         self.$base().test_ref()
//     }
//     }
//  }

trait TestBase {
    type SomeType: Clone;


    fn test1(&self) -> i32;
    //
    // fn test2(&self) -> i32;
    fn test_ref(&self) -> &i32;
}

struct A {
    x: i32
}

impl TestBase for A {
    type SomeType = String;
    fn test1(&self) -> i32 {
        self.x
    }
    // fn test2(&self) -> i32 {
    //     self.x
    // }
    fn test_ref(&self) -> &i32 {
        &self.x
    }
}


trait TestDelegateBase {
    type B: TestBase;

    fn base(&self) -> &Self::B;

    // delegate_base!{base}

    fn test1(&self) -> i32 {
        self.base().test1()
    }

    fn test_ref(&self) -> &i32 {
        self.base().test_ref()
    }
}

struct Aext {
    base: A,
}

impl TestDelegateBase for Aext
{
    type B = A;

    fn base(&self) -> &A {
        &self.base
    }

    fn test1(&self) -> i32 {
        42
    }
}

struct Wrapper<T, B> {
    t: T,
    base_type: PhantomData<B>,
}

trait GetBase {
    type Base;

    fn get_base(&self) -> &Self::Base;
}

impl<T, B> Deref for Wrapper<T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T, B> GetBase for Wrapper<T, B> {
    type Base = T;

    fn get_base(&self) -> &Self::Base {
        &self.t
    }
}


// impl<'a, B, D> TestBase for Wrapper<D, B>
//     where B: TestBase + 'a,
//           D: TestDelegateBase<B=B> + 'a,
// {
//     type SomeType = B::SomeType;
//
//     fn test1(&self) -> i32 {
//         self.base().test1()
//     }
//
//     fn test_ref(&self) -> &i32 {
//         self.base().test_ref()
//     }
// }

// impl<'a, T, B, D> TestBase for T
//     where T: GetBase<Base=D>,
//           B: TestBase + 'a,
//           D: TestDelegateBase<B=B> + 'a,
// {
//     type SomeType = B::SomeType;
//
//     fn test1(&self) -> i32 {
//         self.get_base().test1()
//     }
//
//     fn test_ref(&self) -> &i32 {
//         self.get_base().test_ref()
//     }
// }

#[test]
fn test_delegation() {
    let a = A { x: 7 };
    let a_ext = Aext { base: a };
    assert_eq!(a_ext.test1(), 42);
    assert_eq!(a_ext.test_ref(), &7);
}



///
#[macro_export]
macro_rules! delegate_hierarchy_base {
 ($base:tt) =>

{
    fn cell_by_name(&self, name: &str) -> Option<H::CellId> {
        self.$base().cell_by_name(name)
    }

    fn cell_instance_by_name(&self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
        self.$base().cell_instance_by_name(parent_cell, name)
    }

    fn cell_name(&self, cell: &H::CellId) -> H::NameType {
        self.$base().cell_name(cell)
    }

    fn cell_instance_name(&self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
        self.$base().cell_instance_name(cell_inst)
    }

    fn parent_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
        self.$base().parent_cell(cell_instance)
    }

    fn template_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
        self.$base().template_cell(cell_instance)
    }

    fn for_each_cell<F>(&self, f: F) where F: FnMut(H::CellId) -> () {
        self.$base().for_each_cell(f)
    }

    fn each_cell_vec(&self) -> Vec<H::CellId> {
        self.$base().each_cell_vec()
    }

    fn each_cell(&self) -> Box<dyn Iterator<Item=H::CellId> + '_> {
        self.$base().each_cell()
    }

    fn for_each_cell_instance<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
        self.$base().for_each_cell_instance(cell, f)
    }

    fn each_cell_instance_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
        self.$base().each_cell_instance_vec(cell)
    }

    fn each_cell_instance(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
        self.$base().each_cell_instance(cell)
    }

    fn for_each_cell_dependency<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
        self.$base().for_each_cell_dependency(cell, f)
    }

    fn each_cell_dependency_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
        self.$base().each_cell_dependency_vec(cell)
    }

    fn each_cell_dependency(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
        self.$base().each_cell_dependency(cell)
    }

    fn num_cell_dependencies(&self, cell: &H::CellId) -> usize {
        self.$base().num_cell_dependencies(cell)
    }

    fn for_each_dependent_cell<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
        self.$base().for_each_dependent_cell(cell, f)
    }

    fn each_dependent_cell_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
        self.$base().each_dependent_cell_vec(cell)
    }

    fn each_dependent_cell(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
        self.$base().each_dependent_cell(cell)
    }

    fn num_dependent_cells(&self, cell: &H::CellId) -> usize {
        self.$base().num_dependent_cells(cell)
    }

    fn for_each_cell_reference<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
        self.$base().for_each_cell_reference(cell, f)
    }

    fn each_cell_reference_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
        self.$base().each_cell_reference_vec(cell)
    }

    fn each_cell_reference(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
        self.$base().each_cell_reference(cell)
    }

    fn num_cell_references(&self, cell: &H::CellId) -> usize {
        self.$base().num_cell_references(cell)
    }

    fn num_child_instances(&self, cell: &H::CellId) -> usize {
        self.$base().num_child_instances(cell)
    }

    fn num_cells(&self) -> usize {
        self.$base().num_cells()
    }

    fn get_chip_property(&self, key: &H::NameType) -> Option<PropertyValue> {
        self.$base().get_chip_property(key)
    }

    fn get_cell_property(&self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
        self.$base().get_cell_property(cell, key)
    }

    fn get_cell_instance_property(&self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
        self.$base().get_cell_instance_property(inst, key)
    }
}

}

pub trait HierarchyDelegate<H: HierarchyBase> {

    /// Get a reference to the underlying data structure.
    fn base(&self) -> &H;

    // delegate_hierarchy_base! {base}

    fn cell_by_name(&self, name: &str) -> Option<H::CellId> {
        self.base().cell_by_name(name)
    }

    fn cell_instance_by_name(&self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
        self.base().cell_instance_by_name(parent_cell, name)
    }

    fn cell_name(&self, cell: &H::CellId) -> H::NameType {
        self.base().cell_name(cell)
    }

    fn cell_instance_name(&self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
        self.base().cell_instance_name(cell_inst)
    }

    fn parent_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
        self.base().parent_cell(cell_instance)
    }

    fn template_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
        self.base().template_cell(cell_instance)
    }

    fn for_each_cell<F>(&self, f: F) where F: FnMut(H::CellId) -> () {
        self.base().for_each_cell(f)
    }

    fn each_cell_vec(&self) -> Vec<H::CellId> {
        self.base().each_cell_vec()
    }

    fn each_cell<'a>(&'a self) -> Box<dyn Iterator<Item=H::CellId> + 'a>
        where H: 'a {
        self.base().each_cell()
    }

    fn for_each_cell_instance<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
        self.base().for_each_cell_instance(cell, f)
    }

    fn each_cell_instance_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
        self.base().each_cell_instance_vec(cell)
    }

    fn each_cell_instance<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a>
        where H: 'a {
        self.base().each_cell_instance(cell)
    }

    fn for_each_cell_dependency<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
        self.base().for_each_cell_dependency(cell, f)
    }

    fn each_cell_dependency_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
        self.base().each_cell_dependency_vec(cell)
    }

    fn each_cell_dependency<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a>
        where H: 'a {
        self.base().each_cell_dependency(cell)
    }

    fn num_cell_dependencies(&self, cell: &H::CellId) -> usize {
        self.base().num_cell_dependencies(cell)
    }

    fn for_each_dependent_cell<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
        self.base().for_each_dependent_cell(cell, f)
    }

    fn each_dependent_cell_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
        self.base().each_dependent_cell_vec(cell)
    }

    fn each_dependent_cell<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a>
        where H: 'a {
        self.base().each_dependent_cell(cell)
    }

    fn num_dependent_cells(&self, cell: &H::CellId) -> usize {
        self.base().num_dependent_cells(cell)
    }

    fn for_each_cell_reference<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
        self.base().for_each_cell_reference(cell, f)
    }

    fn each_cell_reference_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
        self.base().each_cell_reference_vec(cell)
    }

    fn each_cell_reference<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a>
        where H: 'a {
        self.base().each_cell_reference(cell)
    }

    fn num_cell_references(&self, cell: &H::CellId) -> usize {
        self.base().num_cell_references(cell)
    }

    fn num_child_instances(&self, cell: &H::CellId) -> usize {
        self.base().num_child_instances(cell)
    }

    fn num_cells(&self) -> usize {
        self.base().num_cells()
    }

    fn get_chip_property(&self, key: &H::NameType) -> Option<PropertyValue> {
        self.base().get_chip_property(key)
    }

    fn get_cell_property(&self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
        self.base().get_cell_property(cell, key)
    }

    fn get_cell_instance_property(&self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
        self.base().get_cell_instance_property(inst, key)
    }
}

impl<'a, H, D> HierarchyBase for Wrapper<D, H>
    where H: HierarchyBase + 'a,
          D: HierarchyDelegate<H> + 'a,

{
    type NameType = H::NameType;
    type CellId = H::CellId;
    type CellInstId = H::CellInstId;

    delegate_hierarchy_base! {base}

    // fn cell_by_name(&self, name: &str) -> Option<H::CellId> {
    //     self.base().cell_by_name(name)
    // }
    //
    // fn cell_instance_by_name(&self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
    //     self.base().cell_instance_by_name(parent_cell, name)
    // }
    //
    // fn cell_name(&self, cell: &H::CellId) -> H::NameType {
    //     self.base().cell_name(cell)
    // }
    //
    // fn cell_instance_name(&self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
    //     self.base().cell_instance_name(cell_inst)
    // }
    //
    // fn parent_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
    //     self.base().parent_cell(cell_instance)
    // }
    //
    // fn template_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
    //     self.base().template_cell(cell_instance)
    // }
    //
    // fn for_each_cell<F>(&self, f: F) where F: FnMut(H::CellId) -> () {
    //     self.base().for_each_cell(f)
    // }
    //
    // fn each_cell_vec(&self) -> Vec<H::CellId> {
    //     self.base().each_cell_vec()
    // }
    //
    // fn each_cell(&self) -> Box<dyn Iterator<Item=H::CellId> + '_> {
    //     self.base().each_cell()
    // }
    //
    // fn for_each_cell_instance<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
    //     self.base().for_each_cell_instance(cell, f)
    // }
    //
    // fn each_cell_instance_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
    //     self.base().each_cell_instance_vec(cell)
    // }
    //
    // fn each_cell_instance(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
    //     self.base().each_cell_instance(cell)
    // }
    //
    // fn for_each_cell_dependency<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
    //     self.base().for_each_cell_dependency(cell, f)
    // }
    //
    // fn each_cell_dependency_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
    //     self.base().each_cell_dependency_vec(cell)
    // }
    //
    // fn each_cell_dependency(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
    //     self.base().each_cell_dependency(cell)
    // }
    //
    // fn num_cell_dependencies(&self, cell: &H::CellId) -> usize {
    //     self.base().num_cell_dependencies(cell)
    // }
    //
    // fn for_each_dependent_cell<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
    //     self.base().for_each_dependent_cell(cell, f)
    // }
    //
    // fn each_dependent_cell_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
    //     self.base().each_dependent_cell_vec(cell)
    // }
    //
    // fn each_dependent_cell(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
    //     self.base().each_dependent_cell(cell)
    // }
    //
    // fn num_dependent_cells(&self, cell: &H::CellId) -> usize {
    //     self.base().num_dependent_cells(cell)
    // }
    //
    // fn for_each_cell_reference<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
    //     self.base().for_each_cell_reference(cell, f)
    // }
    //
    // fn each_cell_reference_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
    //     self.base().each_cell_reference_vec(cell)
    // }
    //
    // fn each_cell_reference(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
    //     self.base().each_cell_reference(cell)
    // }
    //
    // fn num_cell_references(&self, cell: &H::CellId) -> usize {
    //     self.base().num_cell_references(cell)
    // }
    //
    // fn num_child_instances(&self, cell: &H::CellId) -> usize {
    //     self.base().num_child_instances(cell)
    // }
    //
    // fn num_cells(&self) -> usize {
    //     self.base().num_cells()
    // }
    //
    // fn get_chip_property(&self, key: &H::NameType) -> Option<PropertyValue> {
    //     self.base().get_chip_property(key)
    // }
    //
    // fn get_cell_property(&self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
    //     self.base().get_cell_property(cell, key)
    // }
    //
    // fn get_cell_instance_property(&self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
    //     self.base().get_cell_instance_property(inst, key)
    // }
}

