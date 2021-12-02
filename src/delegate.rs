// /*
//  * Copyright (c) 2020-2021 Thomas Kramer.
//  *
//  * This file is part of LibrEDA
//  * (see https://codeberg.org/libreda).
//  *
//  * This program is free software: you can redistribute it and/or modify
//  * it under the terms of the GNU Affero General Public License as
//  * published by the Free Software Foundation, either version 3 of the
//  * License, or (at your option) any later version.
//  *
//  * This program is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  * GNU Affero General Public License for more details.
//  *
//  * You should have received a copy of the GNU Affero General Public License
//  * along with this program. If not, see <http://www.gnu.org/licenses/>.
//  */
//
// use crate::traits::HierarchyBase;
// use crate::prelude::PropertyValue;
// use std::marker::PhantomData;
//
//
// macro_rules! delegate_base {
//  ($base:tt) => {
//     // fn test1(&self) -> i32 {
//     //     self.$base().test1()
//     // }
//     // fn test2(&self) -> i32 {
//     //     self.$base().test2()
//     // }
//     fn test_ref(&self) -> &i32 {
//         self.$base().test_ref()
//     }
//     }
//  }
//
// trait Base {
//     type SomeType: Clone;
//
//
//     // fn test1(&self) -> i32;
//     //
//     // fn test2(&self) -> i32;
//     fn test_ref(&self) -> &i32;
// }
//
// struct A {
//     x: i32
// }
//
// impl Base for A {
//     type SomeType = String;
//     // fn test1(&self) -> i32 {
//     //     self.x
//     // }
//     // fn test2(&self) -> i32 {
//     //     self.x
//     // }
//     fn test_ref(&self) -> &i32 {
//         &self.x
//     }
// }
//
// trait DelegateBase<'a, B: Base + 'a> {
//     fn base(&'a self) -> &'a B;
//     // delegate_base!{base}
//
//     fn test_ref(&'a self) -> &'a i32 {
//         self.base().test_ref()
//     }
// }
//
// struct Wrapper<T, B> {
//     t: T,
//     base_type: PhantomData<B>,
// }
//
// impl<'a, B, D> Base for Wrapper<D, B>
//     where B: Base + 'a,
//           D: DelegateBase<'a, B> {
//     type SomeType = B::SomeType;
//
//     fn test_ref(&'a self) -> &'a i32 {
//         self.t.base().test_ref()
//     }
// }
//
// // impl<T: Base> Base for Wrapper<T> {
// //     type SomeType = T::SomeType;
// //     delegate_base! {base}
// // }
//
// // impl<'a, D, T> Base for Wrapper<D>
// //     where D: DelegateBase<'a, T>,
// //           T: Base {
// //     type SomeType = T::SomeType;
// //     delegate_base! {base}
// // }
//
//
// ///
// #[macro_export]
// macro_rules! delegate_hierarchy_base {
//  ($base:tt) =>
//
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
//     fn each_cell(&self) -> Box<dyn Iterator<Item=H::CellId> + 'a> {
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
//     fn each_cell_instance(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a> {
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
//     fn each_cell_dependency(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a> {
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
//     fn each_dependent_cell(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a> {
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
//     fn each_cell_reference(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a> {
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
// //
// // pub trait HierarchyDelegate<'a, H: HierarchyBase + 'a> {
// //     /// Get a reference to the underlying data structure.
// //     fn base(&'a self) -> &'a H;
// //
// //     // delegate_hierarchy_base! {base}
// //
// //     fn cell_by_name(&'a self, name: &str) -> Option<H::CellId> {
// //         self.base().cell_by_name(name)
// //     }
// //
// //     fn cell_instance_by_name(&'a self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
// //         self.base().cell_instance_by_name(parent_cell, name)
// //     }
// //
// //     fn cell_name(&'a self, cell: &H::CellId) -> H::NameType {
// //         self.base().cell_name(cell)
// //     }
// //
// //     fn cell_instance_name(&'a self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
// //         self.base().cell_instance_name(cell_inst)
// //     }
// //
// //     fn parent_cell(&'a self, cell_instance: &H::CellInstId) -> H::CellId {
// //         self.base().parent_cell(cell_instance)
// //     }
// //
// //     fn template_cell(&'a self, cell_instance: &H::CellInstId) -> H::CellId {
// //         self.base().template_cell(cell_instance)
// //     }
// //
// //     fn for_each_cell<F>(&'a self, f: F) where F: FnMut(H::CellId) -> () {
// //         self.base().for_each_cell(f)
// //     }
// //
// //     fn each_cell_vec(&'a self) -> Vec<H::CellId> {
// //         self.base().each_cell_vec()
// //     }
// //
// //     fn each_cell(&'a self) -> Box<dyn Iterator<Item=H::CellId> + 'a> {
// //         self.base().each_cell()
// //     }
// //
// //     fn for_each_cell_instance<F>(&'a self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
// //         self.base().for_each_cell_instance(cell, f)
// //     }
// //
// //     fn each_cell_instance_vec(&'a self, cell: &H::CellId) -> Vec<H::CellInstId> {
// //         self.base().each_cell_instance_vec(cell)
// //     }
// //
// //     fn each_cell_instance(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a> {
// //         self.base().each_cell_instance(cell)
// //     }
// //
// //     fn for_each_cell_dependency<F>(&'a self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
// //         self.base().for_each_cell_dependency(cell, f)
// //     }
// //
// //     fn each_cell_dependency_vec(&'a self, cell: &H::CellId) -> Vec<H::CellId> {
// //         self.base().each_cell_dependency_vec(cell)
// //     }
// //
// //     fn each_cell_dependency(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a> {
// //         self.base().each_cell_dependency(cell)
// //     }
// //
// //     fn num_cell_dependencies(&'a self, cell: &H::CellId) -> usize {
// //         self.base().num_cell_dependencies(cell)
// //     }
// //
// //     fn for_each_dependent_cell<F>(&'a self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
// //         self.base().for_each_dependent_cell(cell, f)
// //     }
// //
// //     fn each_dependent_cell_vec(&'a self, cell: &H::CellId) -> Vec<H::CellId> {
// //         self.base().each_dependent_cell_vec(cell)
// //     }
// //
// //     fn each_dependent_cell(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + 'a> {
// //         self.base().each_dependent_cell(cell)
// //     }
// //
// //     fn num_dependent_cells(&'a self, cell: &H::CellId) -> usize {
// //         self.base().num_dependent_cells(cell)
// //     }
// //
// //     fn for_each_cell_reference<F>(&'a self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
// //         self.base().for_each_cell_reference(cell, f)
// //     }
// //
// //     fn each_cell_reference_vec(&'a self, cell: &H::CellId) -> Vec<H::CellInstId> {
// //         self.base().each_cell_reference_vec(cell)
// //     }
// //
// //     fn each_cell_reference(&'a self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a> {
// //         self.base().each_cell_reference(cell)
// //     }
// //
// //     fn num_cell_references(&'a self, cell: &H::CellId) -> usize {
// //         self.base().num_cell_references(cell)
// //     }
// //
// //     fn num_child_instances(&'a self, cell: &H::CellId) -> usize {
// //         self.base().num_child_instances(cell)
// //     }
// //
// //     fn num_cells(&'a self) -> usize {
// //         self.base().num_cells()
// //     }
// //
// //     fn get_chip_property(&'a self, key: &H::NameType) -> Option<PropertyValue> {
// //         self.base().get_chip_property(key)
// //     }
// //
// //     fn get_cell_property(&'a self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
// //         self.base().get_cell_property(cell, key)
// //     }
// //
// //     fn get_cell_instance_property(&'a self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
// //         self.base().get_cell_instance_property(inst, key)
// //     }
// // }
// //
// // struct DelegateWrapper<D, BaseType> {
// //     base: D,
// //     base_type: PhantomData<BaseType>,
// // }
// //
// // impl<'a, D, B> DelegateWrapper<D, B> {
// //     fn base(&'a self) -> &'a D {
// //         &self.base
// //     }
// // }
// //
// //
// // impl<'a, H, D> HierarchyBase for DelegateWrapper<D, H>
// //     where H: HierarchyBase + 'a,
// //           D: HierarchyDelegate<'a, H> + 'a,
// //
// //
// // {
// //     type NameType = H::NameType;
// //     type CellId = H::CellId;
// //     type CellInstId = H::CellInstId;
// //
// //     // delegate_hierarchy_base! {base}
// //
// //     fn cell_by_name(&self, name: &str) -> Option<H::CellId> {
// //         self.base().cell_by_name(name)
// //     }
// //
// //     fn cell_instance_by_name(&self, parent_cell: &H::CellId, name: &str) -> Option<H::CellInstId> {
// //         self.base().cell_instance_by_name(parent_cell, name)
// //     }
// //
// //     fn cell_name(&self, cell: &H::CellId) -> H::NameType {
// //         self.base().cell_name(cell)
// //     }
// //
// //     fn cell_instance_name(&self, cell_inst: &H::CellInstId) -> Option<H::NameType> {
// //         self.base().cell_instance_name(cell_inst)
// //     }
// //
// //     fn parent_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
// //         self.base().parent_cell(cell_instance)
// //     }
// //
// //     fn template_cell(&self, cell_instance: &H::CellInstId) -> H::CellId {
// //         self.base().template_cell(cell_instance)
// //     }
// //
// //     fn for_each_cell<F>(&self, f: F) where F: FnMut(H::CellId) -> () {
// //         self.base().for_each_cell(f)
// //     }
// //
// //     fn each_cell_vec(&self) -> Vec<H::CellId> {
// //         self.base().each_cell_vec()
// //     }
// //
// //     fn each_cell(&self) -> Box<dyn Iterator<Item=H::CellId> + '_> {
// //         self.base().each_cell()
// //     }
// //
// //     fn for_each_cell_instance<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
// //         self.base().for_each_cell_instance(cell, f)
// //     }
// //
// //     fn each_cell_instance_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
// //         self.base().each_cell_instance_vec(cell)
// //     }
// //
// //     // fn each_cell_instance(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + 'a> {
// //     //     self.base().each_cell_instance(cell)
// //     // }
// //
// //     fn for_each_cell_dependency<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
// //         self.base().for_each_cell_dependency(cell, f)
// //     }
// //
// //     fn each_cell_dependency_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
// //         self.base().each_cell_dependency_vec(cell)
// //     }
// //
// //     // fn each_cell_dependency(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
// //     //     self.base().each_cell_dependency(cell)
// //     // }
// //
// //     fn num_cell_dependencies(&self, cell: &H::CellId) -> usize {
// //         self.base().num_cell_dependencies(cell)
// //     }
// //
// //     fn for_each_dependent_cell<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellId) -> () {
// //         self.base().for_each_dependent_cell(cell, f)
// //     }
// //
// //     fn each_dependent_cell_vec(&self, cell: &H::CellId) -> Vec<H::CellId> {
// //         self.base().each_dependent_cell_vec(cell)
// //     }
// //
// //     // fn each_dependent_cell(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellId> + '_> {
// //     //     self.base().each_dependent_cell(cell)
// //     // }
// //
// //     fn num_dependent_cells(&self, cell: &H::CellId) -> usize {
// //         self.base().num_dependent_cells(cell)
// //     }
// //
// //     fn for_each_cell_reference<F>(&self, cell: &H::CellId, f: F) where F: FnMut(H::CellInstId) -> () {
// //         self.base().for_each_cell_reference(cell, f)
// //     }
// //
// //     fn each_cell_reference_vec(&self, cell: &H::CellId) -> Vec<H::CellInstId> {
// //         self.base().each_cell_reference_vec(cell)
// //     }
// //     //
// //     // fn each_cell_reference(&self, cell: &H::CellId) -> Box<dyn Iterator<Item=H::CellInstId> + '_> {
// //     //     self.base().each_cell_reference(cell)
// //     // }
// //
// //     fn num_cell_references(&self, cell: &H::CellId) -> usize {
// //         self.base().num_cell_references(cell)
// //     }
// //
// //     fn num_child_instances(&self, cell: &H::CellId) -> usize {
// //         self.base().num_child_instances(cell)
// //     }
// //
// //     fn num_cells(&self) -> usize {
// //         self.base().num_cells()
// //     }
// //
// //     fn get_chip_property(&self, key: &H::NameType) -> Option<PropertyValue> {
// //         self.base().get_chip_property(key)
// //     }
// //
// //     fn get_cell_property(&self, cell: &H::CellId, key: &H::NameType) -> Option<PropertyValue> {
// //         self.base().get_cell_property(cell, key)
// //     }
// //
// //     fn get_cell_instance_property(&self, inst: &H::CellInstId, key: &H::NameType) -> Option<PropertyValue> {
// //         self.base().get_cell_instance_property(inst, key)
// //     }
// // }
// //
