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

use crate::traits::{HierarchyBase, HierarchyEdit};
use crate::prelude::PropertyValue;
//
// ///
// #[macro_export] macro_rules! inherit {
//     ( $($i:ident),* ) => { $( inherit_fn!($i); )* }
// }
//
// ///
// #[macro_export] macro_rules! inherit_fn {
//
//     (cell_by_name) => {
//         fn cell_by_name(&self, name: &str) -> Option<H::CellId> {
//             self.base().cell_by_name(name)
//         }
//     };
//
//     (cell_instance_by_name) => {
//         fn cell_instance_by_name(&self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
//             self.base().cell_instance_by_name(parent_cell, name)
//         }
//     };
//
//     (cell_name) => {
//         fn cell_name(&self, cell: &H::CellId) -> H::NameType {
//             self.base().cell_name(cell)
//         }
//     };
//
//     (cell_instance_name) => {
//         fn cell_instance_name(&self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
//             self.base().cell_instance_name(cell_inst)
//         }
//     };
//
//     (parent_cell) => {
//     fn parent_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
//         self.base().parent_cell(cell_instance)
//     }
//     };
//
//     (template_cell) => {
//     fn template_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
//         self.base().template_cell(cell_instance)
//     }
//     };
//
//     (for_each_cell) => {
//     fn for_each_cell<F>(&self, f: F) where F: FnMut(H::CellId) -> () {
//         self.base().for_each_cell(f)
//     }
//     };
//
//     (each_cell_vec) => {
//     fn each_cell_vec(&self) -> Vec<H::CellId> {
//         self.base().each_cell_vec()
//     }
//     };
//
//     (each_cell) => {
//     fn each_cell<'a>(&'a self) -> Box<dyn Iterator<Item=H::CellId> + 'a>
//         where H: 'a {
//         self.base().each_cell()
//     }
//     };
//
//     (for_each_cell_instance) => {
//     fn for_each_cell_instance<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
//         self.base().for_each_cell_instance(cell, f)
//     }
//     };
//
//     (each_cell_instance_vec) => {
//     fn each_cell_instance_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
//         self.base().each_cell_instance_vec(cell)
//     }
//     };
//
//     (each_cell_instance) => {
//     fn each_cell_instance<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a>
//         where H: 'a {
//         self.base().each_cell_instance(cell)
//     }
//     };
//
//     (for_each_cell_dependency) => {
//     fn for_each_cell_dependency<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
//         self.base().for_each_cell_dependency(cell, f)
//     }
//     };
//
//     (each_cell_dependency_vec) => {
//     fn each_cell_dependency_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
//         self.base().each_cell_dependency_vec(cell)
//     }
//     };
//
//     (each_cell_dependency) => {
//     fn each_cell_dependency<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a>
//         where H: 'a {
//         self.base().each_cell_dependency(cell)
//     }
//     };
//
//     (num_cell_dependencies) => {
//     fn num_cell_dependencies(&self, cell: &H::CellId) -> usize {
//         self.base().num_cell_dependencies(cell)
//     }
//     };
//
//     (for_each_dependent_cell) => {
//     fn for_each_dependent_cell<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
//         self.base().for_each_dependent_cell(cell, f)
//     }
//     };
//
//     (each_dependent_cell_vec) => {
//     fn each_dependent_cell_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
//         self.base().each_dependent_cell_vec(cell)
//     }
//     };
//
//     (each_dependent_cell) => {
//     fn each_dependent_cell<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a>
//         where H: 'a {
//         self.base().each_dependent_cell(cell)
//     }
//     };
//
//     (num_dependent_cells) => {
//     fn num_dependent_cells(&self, cell: &H::CellId) -> usize {
//         self.base().num_dependent_cells(cell)
//     }
//     };
//
//     (for_each_cell_reference) => {
//     fn for_each_cell_reference<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
//         self.base().for_each_cell_reference(cell, f)
//     }
//     };
//
//     (each_cell_reference_vec) => {
//     fn each_cell_reference_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
//         self.base().each_cell_reference_vec(cell)
//     }
//     };
//
//     (each_cell_reference) => {
//     fn each_cell_reference<'a>(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a>
//         where H: 'a {
//         self.base().each_cell_reference(cell)
//     }
//     };
//
//     (num_cell_references) => {
//     fn num_cell_references(&self, cell: &H::CellId) -> usize {
//         self.base().num_cell_references(cell)
//     }
//     };
//
//     (num_child_instances) => {
//     fn num_child_instances(&self, cell: &H::CellId) -> usize {
//         self.base().num_child_instances(cell)
//     }
//     };
//
//     (num_cells) => {
//     fn num_cells(&self) -> usize {
//         self.base().num_cells()
//     }
//     };
//
//     (get_chip_property) => {
//     fn get_chip_property(&self, key: &H::NameType) -> Option<PropertyValue> {
//         self.base().get_chip_property(key)
//     }
//     };
//
//     (get_cell_property) => {
//     fn get_cell_property(&self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
//         self.base().get_cell_property(cell, key)
//     }
//     };
//
//     (get_cell_instance_property) => {
//     fn get_cell_instance_property(&self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
//         self.base().get_cell_instance_property(inst, key)
//     }
//     };
//
// }


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
//
// trait TestBase {
//     type SomeType: Clone;
//
//     fn test1(&self) -> i32;
//     //
//     // fn test2(&self) -> i32;
//     fn test_ref(&self) -> &i32;
// }
//
// struct A {
//     x: i32
// }
//
// impl TestBase for A {
//     type SomeType = String;
//     fn test1(&self) -> i32 {
//         self.x
//     }
//
//     fn test_ref(&self) -> &i32 {
//         &self.x
//     }
// }
//
//
// trait TestDelegateBase {
//     type B: TestBase;
//
//     fn base(&self) -> &Self::B;
//
//     // delegate_base!{base}
//
//     fn test1(&self) -> i32 {
//         self.base().test1()
//     }
//
//     fn test_ref(&self) -> &i32 {
//         self.base().test_ref()
//     }
// }
//
// struct Aext {
//     base: A,
// }
//
// impl TestDelegateBase for Aext
// {
//     type B = A;
//
//     fn base(&self) -> &A {
//         &self.base
//     }
//
//     fn test1(&self) -> i32 {
//         42
//     }
// }
//
//
// impl<'a, T, B> TestBase for T
//     where
//         T: TestDelegateBase<B=B>,
//         B: TestBase + 'a,
// {
//     type SomeType = B::SomeType;
//
//     fn test1(&self) -> i32 {
//         self.test1()
//     }
//
//     fn test_ref(&self) -> &i32 {
//         self.test_ref()
//     }
// }
//
// #[test]
// fn test_delegation() {
//     let a = A { x: 7 };
//     let a_ext = Aext { base: a };
//
//     assert_eq!(TestDelegateBase::test1(&a_ext), 42);
//     assert_eq!(TestBase::test1(&a_ext), 42);
//     // assert_eq!(a_ext.test_ref(), &7);
//
//     fn use_testbase<T: TestBase>(t: &T) {
//         assert_eq!(t.test1(), 42);
//         assert_eq!(t.test_ref(), &7);
//     }
//
//     use_testbase(&a_ext);
// }


//
// ///
// #[macro_export]
// macro_rules! delegate_hierarchy_base {
//  ($base:tt) =>
// {
//     fn cell_by_name(&self, name: &str) -> Option<H::CellId> {
//         self.$base().cell_by_name(name)
//     }
//
//     fn cell_instance_by_name(&self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
//         self.$base().cell_instance_by_name(parent_cell, name)
//     }
//
//     fn cell_name(&self, cell: &H::CellId) -> H::NameType {
//         self.$base().cell_name(cell)
//     }
//
//     fn cell_instance_name(&self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
//         self.$base().cell_instance_name(cell_inst)
//     }
//
//     fn parent_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
//         self.$base().parent_cell(cell_instance)
//     }
//
//     fn template_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
//         self.$base().template_cell(cell_instance)
//     }
//
//     fn for_each_cell<F>(&self, f: F) where F: FnMut(H::CellId) -> () {
//         self.$base().for_each_cell(f)
//     }
//
//     fn each_cell_vec(&self) -> Vec<H::CellId> {
//         self.$base().each_cell_vec()
//     }
//
//     fn each_cell(&self) -> Box<dyn Iterator<Item=H::CellId> + '_> {
//         self.$base().each_cell()
//     }
//
//     fn for_each_cell_instance<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
//         self.$base().for_each_cell_instance(cell, f)
//     }
//
//     fn each_cell_instance_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
//         self.$base().each_cell_instance_vec(cell)
//     }
//
//     fn each_cell_instance(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
//         self.$base().each_cell_instance(cell)
//     }
//
//     fn for_each_cell_dependency<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
//         self.$base().for_each_cell_dependency(cell, f)
//     }
//
//     fn each_cell_dependency_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
//         self.$base().each_cell_dependency_vec(cell)
//     }
//
//     fn each_cell_dependency(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
//         self.$base().each_cell_dependency(cell)
//     }
//
//     fn num_cell_dependencies(&self, cell: &H::CellId) -> usize {
//         self.$base().num_cell_dependencies(cell)
//     }
//
//     fn for_each_dependent_cell<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
//         self.$base().for_each_dependent_cell(cell, f)
//     }
//
//     fn each_dependent_cell_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
//         self.$base().each_dependent_cell_vec(cell)
//     }
//
//     fn each_dependent_cell(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
//         self.$base().each_dependent_cell(cell)
//     }
//
//     fn num_dependent_cells(&self, cell: &H::CellId) -> usize {
//         self.$base().num_dependent_cells(cell)
//     }
//
//     fn for_each_cell_reference<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
//         self.$base().for_each_cell_reference(cell, f)
//     }
//
//     fn each_cell_reference_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
//         self.$base().each_cell_reference_vec(cell)
//     }
//
//     fn each_cell_reference(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
//         self.$base().each_cell_reference(cell)
//     }
//
//     fn num_cell_references(&self, cell: &H::CellId) -> usize {
//         self.$base().num_cell_references(cell)
//     }
//
//     fn num_child_instances(&self, cell: &H::CellId) -> usize {
//         self.$base().num_child_instances(cell)
//     }
//
//     fn num_cells(&self) -> usize {
//         self.$base().num_cells()
//     }
//
//     fn get_chip_property(&self, key: &H::NameType) -> Option<PropertyValue> {
//         self.$base().get_chip_property(key)
//     }
//
//     fn get_cell_property(&self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
//         self.$base().get_cell_property(cell, key)
//     }
//
//     fn get_cell_instance_property(&self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
//         self.$base().get_cell_instance_property(inst, key)
//     }
// }
//
// }

/// Define the same functions as [`HierarchyBase`] but just prepend a `d_` to
/// avoid naming conflicts.
/// The default implementation just forwards the call to the `base()`.
/// This allows to selectively re-implement some functions or fully delegate
/// the trait to an attribute of a struct.
pub trait HierarchyBaseDecorator
    where Self: Sized
{
    type H: HierarchyBase;

    /// Get a reference to the underlying data structure.
    fn base(&self) -> &Self::H;

    fn d_cell_by_name(&self, name: &str) -> Option<<Self::H as HierarchyBase>::CellId> {
        self.base().cell_by_name(name)
    }

    fn d_cell_instance_by_name(&self, parent_cell: &<Self::H as HierarchyBase>::CellId, name: &str) -> Option<<Self::H as HierarchyBase>::CellInstId> {
        self.base().cell_instance_by_name(parent_cell, name)
    }

    fn d_cell_name(&self, cell: &<Self::H as HierarchyBase>::CellId) -> <Self::H as HierarchyBase>::NameType {
        self.base().cell_name(cell)
    }

    fn d_cell_instance_name(&self, cell_inst: &<Self::H as HierarchyBase>::CellInstId) -> Option<<Self::H as HierarchyBase>::NameType> {
        self.base().cell_instance_name(cell_inst)
    }

    fn d_parent_cell(&self, cell_instance: &<Self::H as HierarchyBase>::CellInstId) -> <Self::H as HierarchyBase>::CellId {
        self.base().parent_cell(cell_instance)
    }

    fn d_template_cell(&self, cell_instance: &<Self::H as HierarchyBase>::CellInstId) -> <Self::H as HierarchyBase>::CellId {
        self.base().template_cell(cell_instance)
    }

    fn d_for_each_cell<F>(&self, f: F) where F: FnMut(<Self::H as HierarchyBase>::CellId) -> () {
        self.base().for_each_cell(f)
    }

    fn d_each_cell_vec(&self) -> Vec<<Self::H as HierarchyBase>::CellId> {
        self.base().each_cell_vec()
    }

    fn d_each_cell(&self) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellId> + '_> {
        self.base().each_cell()
    }

    fn d_for_each_cell_instance<F>(&self, cell: &<Self::H as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::H as HierarchyBase>::CellInstId) -> () {
        self.base().for_each_cell_instance(cell, f)
    }

    fn d_each_cell_instance_vec(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Vec<<Self::H as HierarchyBase>::CellInstId> {
        self.base().each_cell_instance_vec(cell)
    }

    fn d_each_cell_instance(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellInstId> + '_> {
        self.base().each_cell_instance(cell)
    }

    fn d_for_each_cell_dependency<F>(&self, cell: &<Self::H as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::H as HierarchyBase>::CellId) -> () {
        self.base().for_each_cell_dependency(cell, f)
    }

    fn d_each_cell_dependency_vec(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Vec<<Self::H as HierarchyBase>::CellId> {
        self.base().each_cell_dependency_vec(cell)
    }

    fn d_each_cell_dependency(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellId> + '_> {
        self.base().each_cell_dependency(cell)
    }

    fn d_num_cell_dependencies(&self, cell: &<Self::H as HierarchyBase>::CellId) -> usize {
        self.base().num_cell_dependencies(cell)
    }

    fn d_for_each_dependent_cell<F>(&self, cell: &<Self::H as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::H as HierarchyBase>::CellId) -> () {
        self.base().for_each_dependent_cell(cell, f)
    }

    fn d_each_dependent_cell_vec(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Vec<<Self::H as HierarchyBase>::CellId> {
        self.base().each_dependent_cell_vec(cell)
    }

    fn d_each_dependent_cell(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellId> + '_> {
        self.base().each_dependent_cell(cell)
    }

    fn d_num_dependent_cells(&self, cell: &<Self::H as HierarchyBase>::CellId) -> usize {
        self.base().num_dependent_cells(cell)
    }

    fn d_for_each_cell_reference<F>(&self, cell: &<Self::H as HierarchyBase>::CellId, f: F) where F: FnMut(<Self::H as HierarchyBase>::CellInstId) -> () {
        self.base().for_each_cell_reference(cell, f)
    }

    fn d_each_cell_reference_vec(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Vec<<Self::H as HierarchyBase>::CellInstId> {
        self.base().each_cell_reference_vec(cell)
    }

    fn d_each_cell_reference(&self, cell: &<Self::H as HierarchyBase>::CellId) -> Box<dyn Iterator<Item=<Self::H as HierarchyBase>::CellInstId> + '_> {
        self.base().each_cell_reference(cell)
    }

    fn d_num_cell_references(&self, cell: &<Self::H as HierarchyBase>::CellId) -> usize {
        self.base().num_cell_references(cell)
    }

    fn d_num_child_instances(&self, cell: &<Self::H as HierarchyBase>::CellId) -> usize {
        self.base().num_child_instances(cell)
    }

    fn d_num_cells(&self) -> usize {
        self.base().num_cells()
    }

    fn d_get_chip_property(&self, key: &<Self::H as HierarchyBase>::NameType) -> Option<PropertyValue> {
        self.base().get_chip_property(key)
    }

    fn d_get_cell_property(&self, cell: &<Self::H as HierarchyBase>::CellId, key: &<Self::H as HierarchyBase>::NameType) -> Option<PropertyValue> {
        self.base().get_cell_property(cell, key)
    }

    fn d_get_cell_instance_property(&self, inst: &<Self::H as HierarchyBase>::CellInstId, key: &<Self::H as HierarchyBase>::NameType) -> Option<PropertyValue> {
        self.base().get_cell_instance_property(inst, key)
    }
}


impl<'a, T, H> HierarchyBase for T
    where
        T: HierarchyBaseDecorator<H=H>,
        H: HierarchyBase + 'a,
{
    type NameType = H::NameType;
    type CellId = H::CellId;
    type CellInstId = H::CellInstId;

    fn cell_by_name(&self, name: &str) -> Option<Self::CellId> {
        self.d_cell_by_name(name)
    }

    fn cell_instance_by_name(&self, parent_cell: &Self::CellId, name: &str) -> Option<Self::CellInstId> {
        self.d_cell_instance_by_name(parent_cell, name)
    }

    fn cell_name(&self, cell: &Self::CellId) -> Self::NameType {
        self.d_cell_name(cell)
    }

    fn cell_instance_name(&self, cell_inst: &Self::CellInstId) -> Option<Self::NameType> {
        self.d_cell_instance_name(cell_inst)
    }

    fn parent_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.d_parent_cell(cell_instance)
    }

    fn template_cell(&self, cell_instance: &Self::CellInstId) -> Self::CellId {
        self.d_template_cell(cell_instance)
    }

    fn for_each_cell<F>(&self, f: F) where F: FnMut(Self::CellId) -> () {
        self.d_for_each_cell(f)
    }

    fn each_cell_vec(&self) -> Vec<Self::CellId> {
        self.d_each_cell_vec()
    }

    fn each_cell(&self) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        self.d_each_cell()
    }

    fn for_each_cell_instance<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.d_for_each_cell_instance(cell, f)
    }

    fn each_cell_instance_vec(&self, cell: &Self::CellId) -> Vec<Self::CellInstId> {
        self.d_each_cell_instance_vec(cell)
    }

    fn each_cell_instance(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        self.d_each_cell_instance(cell)
    }

    fn for_each_cell_dependency<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.d_for_each_cell_dependency(cell, f)
    }

    fn each_cell_dependency_vec(&self, cell: &Self::CellId) -> Vec<Self::CellId> {
        self.d_each_cell_dependency_vec(cell)
    }

    fn each_cell_dependency(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        self.d_each_cell_dependency(cell)
    }

    fn num_cell_dependencies(&self, cell: &Self::CellId) -> usize {
        self.d_num_cell_dependencies(cell)
    }

    fn for_each_dependent_cell<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellId) -> () {
        self.d_for_each_dependent_cell(cell, f)
    }

    fn each_dependent_cell_vec(&self, cell: &Self::CellId) -> Vec<Self::CellId> {
        self.d_each_dependent_cell_vec(cell)
    }

    fn each_dependent_cell(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellId> + '_> {
        self.d_each_dependent_cell(cell)
    }

    fn num_dependent_cells(&self, cell: &Self::CellId) -> usize {
        self.d_num_dependent_cells(cell)
    }

    fn for_each_cell_reference<F>(&self, cell: &Self::CellId, f: F) where F: FnMut(Self::CellInstId) -> () {
        self.d_for_each_cell_reference(cell, f)
    }

    fn each_cell_reference_vec(&self, cell: &Self::CellId) -> Vec<Self::CellInstId> {
        self.d_each_cell_reference_vec(cell)
    }

    fn each_cell_reference(&self, cell: &Self::CellId) -> Box<dyn Iterator<Item=Self::CellInstId> + '_> {
        self.d_each_cell_reference(cell)
    }

    fn num_cell_references(&self, cell: &Self::CellId) -> usize {
        self.d_num_cell_references(cell)
    }

    fn num_child_instances(&self, cell: &Self::CellId) -> usize {
        self.d_num_child_instances(cell)
    }

    fn num_cells(&self) -> usize {
        self.d_num_cells()
    }

    fn get_chip_property(&self, key: &Self::NameType) -> Option<PropertyValue> {
        self.d_get_chip_property(key)
    }

    fn get_cell_property(&self, cell: &Self::CellId, key: &Self::NameType) -> Option<PropertyValue> {
        self.d_get_cell_property(cell, key)
    }

    fn get_cell_instance_property(&self, inst: &Self::CellInstId, key: &Self::NameType) -> Option<PropertyValue> {
        self.d_get_cell_instance_property(inst, key)
    }
}

pub trait HierarchyEditDecorator
    where Self: Sized
{
    type H: HierarchyEdit;

    /// Get a reference to the underlying data structure.
    fn mut_base(&mut self) -> &mut Self::H;

    fn d_new() -> Self;

    fn d_create_cell(&mut self, name: <Self::H as HierarchyBase>::NameType) -> <Self::H as HierarchyBase>::CellId {
        self.mut_base().create_cell(name)
    }

    fn d_remove_cell(&mut self, cell_id: &<Self::H as HierarchyBase>::CellId) {
        self.mut_base().remove_cell(cell_id)
    }

    fn d_create_cell_instance(&mut self,
                              parent_cell: &<Self::H as HierarchyBase>::CellId,
                              template_cell: &<Self::H as HierarchyBase>::CellId,
                              name: Option<<Self::H as HierarchyBase>::NameType>) -> <Self::H as HierarchyBase>::CellInstId {
        self.mut_base().create_cell_instance(parent_cell, template_cell, name)
    }


    fn d_remove_cell_instance(&mut self, inst: &<Self::H as HierarchyBase>::CellInstId) {
        self.mut_base().remove_cell_instance(inst)
    }

    fn d_rename_cell_instance(&mut self, inst: &<Self::H as HierarchyBase>::CellInstId, new_name: Option<<Self::H as HierarchyBase>::NameType>) {
        self.mut_base().rename_cell_instance(inst, new_name)
    }

    fn d_rename_cell(&mut self, cell: &<Self::H as HierarchyBase>::CellId, new_name: <Self::H as HierarchyBase>::NameType) {
        self.mut_base().rename_cell(cell, new_name)
    }

    fn d_set_chip_property(&mut self, key: <Self::H as HierarchyBase>::NameType, value: PropertyValue) {
        self.mut_base().set_chip_property(key, value)
    }

    fn d_set_cell_property(&mut self, cell: &<Self::H as HierarchyBase>::CellId, key: <Self::H as HierarchyBase>::NameType, value: PropertyValue) {
        self.mut_base().set_cell_property(cell, key, value)
    }

    fn d_set_cell_instance_property(&mut self, inst: &<Self::H as HierarchyBase>::CellInstId, key: <Self::H as HierarchyBase>::NameType, value: PropertyValue) {
        self.mut_base().set_cell_instance_property(inst, key, value)
    }
}

impl<'a, T, H> HierarchyEdit for T
    where
        T: HierarchyBase<NameType=H::NameType, CellId=H::CellId, CellInstId=H::CellInstId>
        + HierarchyEditDecorator<H=H>,
        H: HierarchyEdit + 'a {
    fn new() -> Self {
        Self::d_new()
    }

    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId {
        self.d_create_cell(name)
    }

    fn remove_cell(&mut self, cell_id: &Self::CellId) {
        self.d_remove_cell(cell_id)
    }

    fn create_cell_instance(&mut self, parent_cell: &Self::CellId, template_cell: &Self::CellId, name: Option<Self::NameType>) -> Self::CellInstId {
        self.d_create_cell_instance(parent_cell, template_cell, name)
    }

    fn remove_cell_instance(&mut self, inst: &Self::CellInstId) {
        self.d_remove_cell_instance(inst)
    }

    fn rename_cell_instance(&mut self, inst: &Self::CellInstId, new_name: Option<Self::NameType>) {
        self.d_rename_cell_instance(inst, new_name)
    }

    fn rename_cell(&mut self, cell: &Self::CellId, new_name: Self::NameType) {
        self.d_rename_cell(cell, new_name)
    }

    fn set_chip_property(&mut self, key: Self::NameType, value: PropertyValue) {
        self.d_set_chip_property(key, value)
    }

    fn set_cell_property(&mut self, cell: &Self::CellId, key: Self::NameType, value: PropertyValue) {
        self.d_set_cell_property(cell, key, value)
    }

    fn set_cell_instance_property(&mut self, inst: &Self::CellInstId, key: Self::NameType, value: PropertyValue) {
        self.d_set_cell_instance_property(inst, key, value)
    }
}

#[test]
fn test_hierarchy_decorator() {
    use crate::chip::Chip;
    let chip = Chip::new();

    struct DummyDecorator<T>(T);

    impl<H: HierarchyBase> HierarchyBaseDecorator for DummyDecorator<H> {
        type H = H;

        fn base(&self) -> &Self::H {
            &self.0
        }
    }

    impl<H: HierarchyEdit> HierarchyEditDecorator for DummyDecorator<H> {
        type H = H;

        fn mut_base(&mut self) -> &mut Self::H {
            &mut self.0
        }

        fn d_new() -> Self {
            Self(H::new())
        }
    }

    let mut decorated_chip = DummyDecorator(chip);
    // Read access should work.
    decorated_chip.num_cells();
    // Editing should work.
    decorated_chip.create_cell("A".into());
}