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

//! Data structure for efficient representation of hierarchical templates and
//! fly-weight instances thereof.
//!
//! This is mainly used to represent netlists and hierarchical layouts.

use crate::index::*;

use itertools::Itertools;

// Use an alternative hasher that has good performance for integer keys.
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::HashMap;
use crate::rc_string::RcString;
use iron_shapes::point::Deref;
use std::ops::DerefMut;
use std::marker::PhantomData;
use std::fmt;

type IntHashMap<K, V> = FnvHashMap<K, V>;
type IntHashSet<V> = FnvHashSet<V>;

#[derive(Debug, Clone)]
pub struct FlyWeightContainer<T, I> {
    /// All templates indexed by ID.
    templates: IntHashMap<Index<T>, T>,
    /// All instances indexed by ID.
    instances: IntHashMap<Index<I>, I>,
    /// Counter for generating the template IDs.
    template_id_generator: IndexGenerator<T>,
    /// Counter for generating the instance IDs.
    instance_id_generator: IndexGenerator<I>,
    /// Lookup table for finding templates by name.
    templates_by_name: HashMap<RcString, Index<T>>,
}

pub trait FlyWeightContainerTrait<T, I>
    where T: TemplateTrait<T, I> {
    fn fwc(&self) -> &FlyWeightContainer<T, I>;
    fn fwc_mut(&mut self) -> &mut FlyWeightContainer<T, I>;

    /// Callback function that will be used to clean up
    /// when calling `remove_template()`.
    fn destroy_template(&mut self, _template_id: &Index<T>) {}

    /// Callback function that will be used to clean up
    /// when calling `remove_instance()`.
    fn destroy_instance(&mut self, _instance_id: &Index<I>) {}
}

macro_rules! impl_flyweight_container {
    ($T:ty, $I:ty) => {

    /// Get a reference to the template with the given `ID`.
    ///
    /// # Panics
    /// Panics if the ID does not exist.
    pub fn template_by_id(&self, id: &Index<$T>) -> TemplateRef<Self, $T, $I> {
        let template = &self.fwc().templates[id];
        TemplateRef::new(
            self,
            template,
        )
    }

    /// Find the ID of the template with the given `name`.
    pub fn template_id_by_name(&self, name: &str) -> Option<Index<$T>> {
        self.fwc().templates_by_name.get(name).copied()
    }

    /// Get a reference to the template with the given name.
    pub fn template_by_name(&self, name: &str) -> Option<TemplateRef<'_, Self, $T, $I>> {
        let template_id = self.template_id_by_name(name);

        template_id.map(|template_id| {
            let template = &self.fwc().templates[&template_id];
            TemplateRef::new(self, template)
        })
    }

    /// Iterate over all templates in this layout.
    pub fn each_template(&self) -> impl Iterator<Item=TemplateRef<'_, Self, $T, $I>> + ExactSizeIterator {
        self.fwc().templates.values()
            .map(move |template| TemplateRef::new(self, template))
    }

    pub fn create_template(&mut self, name: RcString) -> Index<$T> {
        assert!(!self.fwc().templates_by_name.contains_key(&name), "Template with this name already exists.");
        let id = self.fwc_mut().template_id_generator.next();

        let template = Template {
            name: name.clone(),
            id: id,
            child_instances: Default::default(),
            child_instances_by_name: Default::default(),
            template_references: Default::default(),
            dependencies: Default::default(),
            dependent_templates: Default::default(),
        };

        self.fwc_mut().templates.insert(id, <$T>::new(template));
        self.fwc_mut().templates_by_name.insert(name, id);

        id
    }

    pub fn remove_template(&mut self, template_id: &Index<$T>) {

        // Give the container implementation the chance to do some clean-up.
        self.destroy_template(template_id);

        // Remove all instances inside this template.
        let instances = self.fwc().templates[template_id].tpl().child_instances.iter().copied().collect_vec();
        for inst in instances {
            self.remove_instance(&inst);
        }
        // Remove all instances of this cell.
        let references = self.fwc().templates[template_id].tpl().template_references.iter().copied().collect_vec();
        for inst in references {
            self.remove_instance(&inst);
        }

        // Remove the cell.
        let name = self.fwc().templates[template_id].tpl().name.clone();
        self.fwc_mut().templates_by_name.remove(&name).unwrap();
        self.fwc_mut().templates.remove(&template_id).unwrap();
    }

    pub fn create_instance(&mut self, parent_id: &Index<$T>,
                           template_id: &Index<$T>,
                           name: RcString) -> Index<$I> {
        let id = self.fwc_mut().instance_id_generator.next();

        {
            // Check that creating this instance does not create a cycle in the dependency graph.
            // There can be no recursive instances.
            let mut stack: Vec<Index<_>> = vec![*parent_id];
            while let Some(c) = stack.pop() {
                if &c == template_id {
                    // The cell to be instantiated depends on the current template.
                    // This would insert a loop into the dependency tree.
                    // TODO: Don't panic but return an `Err`.
                    panic!("Cannot create recursive instances.");
                }
                // Follow the dependent templates wards the root.
                stack.extend(self.fwc().templates[&c].tpl().dependent_templates.keys().copied())
            }
        }

        let inst = Instance {
            name: name.clone(),
            parent_id: *parent_id,
            id: id,
            template_id: *template_id,
        };

        self.fwc_mut().instances.insert(id, <$I>::new(inst));
        self.fwc_mut().templates.get_mut(parent_id).unwrap()
            .tpl_mut().child_instances.insert(id);
        self.fwc_mut().templates.get_mut(template_id).unwrap()
            .tpl_mut().template_references.insert(id);

        {
            debug_assert!(!self.fwc().templates[parent_id].tpl().child_instances_by_name.contains_key(&name),
                          "Instance name already exists.");
            self.fwc_mut().templates.get_mut(parent_id).unwrap()
            .tpl_mut().child_instances_by_name.insert(name, id);
        }

        // Remember dependency.
        {
            self.fwc_mut().templates.get_mut(parent_id).unwrap()
                .tpl_mut().dependencies.entry(*template_id)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        // Remember dependency.
        {
            self.fwc_mut().templates.get_mut(template_id).unwrap()
                .tpl_mut().dependent_templates.entry(*parent_id)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        id
    }

    /// Remove an instance.
    pub fn remove_instance(&mut self, instance_id: &Index<$I>) {

        // Give the container implementation the chance to do some clean-up.
        self.destroy_instance(instance_id);

        // Remove the instance and all references.
        let parent = self.fwc().instances[instance_id].inst().parent_id;
        let template = self.fwc().instances[instance_id].inst().template_id;

        // Remove dependency.
        {
            // Decrement counter.
            let parent_mut = self.fwc_mut().templates.get_mut(&parent).unwrap().tpl_mut();
            let count = parent_mut.dependencies.entry(template)
                .or_insert(0); // Should not happen.
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                parent_mut.dependencies.remove(&template);
            }
        }

        // Remove dependency.
        {
            // Decrement counter.
            let template_mut = self.fwc_mut().templates.get_mut(&template).unwrap().tpl_mut();
            let count = template_mut.dependent_templates.entry(parent)
                .or_insert(0); // Should not happen.
            *count -= 1;

            if *count == 0 {
                // Remove entry.
                template_mut.dependent_templates.remove(&parent);
            }
        }

        self.fwc_mut().instances.remove(&instance_id).unwrap();
        self.fwc_mut().templates.get_mut(&parent).unwrap().tpl_mut().child_instances.remove(instance_id);
        self.fwc_mut().templates.get_mut(&template).unwrap().tpl_mut().child_instances.remove(instance_id);
    }
    }
}

pub trait TemplateTrait<T, I> {
    fn tpl(&self) -> &Template<T, I>;
    fn tpl_mut(&mut self) -> &mut Template<T, I>;

    /// Create a new default instance with a struct `t` holding references.
    /// `t` must be made accessible with the `tpl()` and `tpl_mut()` functions.
    fn new(t: Template<T, I>) -> Self;
}

macro_rules! impl_template {
    ($T:ty, $I:ty) => {

        /// Get the name of this template.
        pub fn name(&self) -> &RcString {
            &self.tpl().name
        }

        /// Get the ID of this template.
        /// The ID uniquely identifies the template within this layout.
        pub fn id(&self) -> Index<$T> {
            self.tpl().id
        }

        /// Find a child instance in this template by its name.
        pub fn instance_id_by_name(&self, name: &str) -> Option<Index<$I>> {
            self.tpl().child_instances_by_name.get(name).copied()
        }


        /// Iterate over the IDs of the child template instances.
        pub fn each_instance_id(&self) -> impl Iterator<Item=Index<$I>> + ExactSizeIterator + '_ {
            self.tpl().child_instances.iter().copied()
        }

        /// Iterate over the IDs of each dependency of this template.
        /// A dependency is a template that is instantiated in `self`.
        pub fn each_dependency_id(&self) -> impl Iterator<Item=Index<$T>> + ExactSizeIterator + '_ {
            self.tpl().dependencies.keys().copied()
        }

        /// Iterate over the IDs of templates that depends on this template.
        pub fn each_dependent_template_id(&self) -> impl Iterator<Item=Index<$T>> + ExactSizeIterator + '_ {
            self.tpl().dependent_templates.keys().copied()
        }

    }
}

pub trait InstanceTrait<T, I> {
    fn inst(&self) -> &Instance<T, I>;
    fn inst_mut(&mut self) -> &mut Instance<T, I>;

    fn new(i: Instance<T, I>) -> Self;

    // /// Get the name of this instance.
    // fn name(&self) -> &'_ RcString {
    //     &self.inst().name
    // }

    /// Get the name of this instance.
    /// TODO: Return reference instead of cloned value.
    fn name(&self) -> RcString {
        self.inst().name.clone()
    }

    /// Get the ID of this instance.
    /// The ID uniquely identifies the cell within its container.
    fn id(&self) -> Index<I> {
        self.inst().id
    }

    /// Get the ID of the parent template.
    fn parent_id(&self) -> Index<T> {
        self.inst().parent_id
    }

    /// Get the ID of the template of this instance.
    fn template_id(&self) -> Index<T> {
        self.inst().template_id
    }
}

impl<T, I> Default for FlyWeightContainer<T, I> {
    fn default() -> Self {
        Self {
            templates: Default::default(),
            instances: Default::default(),
            template_id_generator: Default::default(),
            instance_id_generator: Default::default(),
            templates_by_name: Default::default(),
        }
    }
}

/// Data structure for all indices and information that a flyweight template must hold.
#[derive(Clone, Debug)]
pub struct Template<T, I> {
    /// Template name.
    name: RcString,
    /// The index of this template inside the container.
    id: Index<T>,
    /// Child instances.
    child_instances: IntHashSet<Index<I>>,

    /// Cell instances indexed by name.
    child_instances_by_name: HashMap<RcString, Index<I>>,

    /// All the instances of this template.
    template_references: IntHashSet<Index<I>>,

    /// Set of templates that are dependencies of this template.
    /// Stored together with a counter of how many instances of the dependency are present.
    /// This are the templates towards the leaves in the dependency tree.
    dependencies: IntHashMap<Index<T>, usize>,
    /// Templates that use an instance of this template.
    /// This are the templates towards the root in the dependency tree.
    dependent_templates: IntHashMap<Index<T>, usize>,
}

/// A 'fat' reference to a template.
///
/// This struct keeps a reference to a template as well as a reference to the container.
///
/// This allows convenient read-only access.
#[derive(Clone)]
pub struct TemplateRef<'a, C: ?Sized, T: ?Sized, I: ?Sized>
{
    /// Reference to the parent layout.
    container: &'a C,
    /// Reference to the template.
    template: &'a T,
    instance_type: PhantomData<I>,
}


impl<'a, C: ?Sized, T, I> fmt::Debug for TemplateRef<'a, C, T, I>
    where T: TemplateTrait<T, I>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TemplateRef({})", self.template.tpl().id)
    }
}

impl<'a, C: ?Sized, T: ?Sized, I: ?Sized> TemplateRef<'a, C, T, I>
{
    fn new(container: &'a C, template: &'a T) -> Self {
        Self {
            container,
            template,
            instance_type: PhantomData,
        }
    }
}

impl<'a, C, T, I> Deref for TemplateRef<'a, C, T, I>
    where C: FlyWeightContainerTrait<T, I>,
          T: TemplateTrait<T, I> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.template
    }
}

impl<'a, C, T, I> TemplateRef<'a, C, T, I>
    where C: FlyWeightContainerTrait<T, I>,
          T: TemplateTrait<T, I> {
    pub fn id(&self) -> Index<T> {
        self.tpl().id
    }

    /// Iterate over all instances in this template.
    pub fn each_instance_ref(&self) -> impl Iterator<Item=InstanceRef<'_, C, T, I>> + ExactSizeIterator {
        self.tpl().child_instances.iter()
            .map(move |inst_id| {
                let inst = &self.container.fwc().instances[inst_id];
                InstanceRef::new(
                    self.container,
                    inst,
                )
            })
    }

    /// Find a child instance by its name.
    /// Returns `None` if no such instance exists.
    pub fn instance_ref_by_name(&self, name: &str) -> Option<InstanceRef<'_, C, T, I>> {
        // let id = self.instance_id_by_name(name);
        let id = self.tpl().child_instances_by_name.get(name).copied();
        id.map(|id| {
            let inst = &self.container.fwc().instances[&id];
            InstanceRef::new(self.container, inst)
        })
    }

    /// Iterate over the references to all cells that are dependencies of this cell.
    pub fn each_dependency_ref(&self) -> impl Iterator<Item=TemplateRef<'_, C, T, I>> + ExactSizeIterator {
        self.tpl().dependencies.keys().copied()
            .map(move |id| TemplateRef::new(
                self.container,
                &self.container.fwc().templates[&id],
            ))
    }

    /// Iterate over the references to all cells that are dependent on this cell.
    pub fn each_dependent_template_ref(&self) -> impl Iterator<Item=TemplateRef<'_, C, T, I>> + ExactSizeIterator {
        self.tpl().dependent_templates.keys().copied()
            .map(move |id| TemplateRef::new(
                self.container,
                &self.container.fwc().templates[&id],
            ))
    }
}

/// An actual instance of a template.
#[derive(Clone, Debug)]
pub struct Instance<T, I> {
    /// Name of the instance.
    name: RcString,
    /// ID of the parent template.
    parent_id: Index<T>,
    /// Identifier. Uniquely identifies the instance within the parent template.
    id: Index<I>,
    /// ID of the template cell.
    template_id: Index<T>,
}

macro_rules! impl_instance {
    ($T:ty, $I:ty) => {
        /// Get the name of this instance.
        pub fn name(&self) -> &RcString {
            &self.inst().name
        }

        /// Get the ID of this instance.
        /// The ID uniquely identifies the cell within its container.
        pub fn id(&self) -> Index<$I> {
            self.inst().id
        }

        /// Get the ID of the parent template.
        pub fn parent_id(&self) -> Index<$T> {
            self.inst().parent_id
        }

        /// Get the ID of the template of this instance.
        pub fn template_id(&self) -> Index<$T> {
            self.inst().template_id
        }
    }
}

/// A reference to an instance.
///
/// This struct also keeps a reference to the container struct.
#[derive(Clone)]
pub struct InstanceRef<'a, C: ?Sized, T: ?Sized, I: ?Sized> {
    container: &'a C,
    inst: &'a I,
    template_type: PhantomData<T>,
}


impl<'a, C, T, I> InstanceRef<'a, C, T, I>
    where C: FlyWeightContainerTrait<T, I>,
          T: TemplateTrait<T, I> {
    fn new(container: &'a C, inst: &'a I) -> Self {
        Self {
            container,
            inst,
            template_type: PhantomData,
        }
    }
}

impl<'a, C: ?Sized, T, I> fmt::Debug for InstanceRef<'a, C, T, I>
    where I: InstanceTrait<T, I>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InstanceRef({})", self.inst.id())
    }
}

impl<'a, C, T, I> Deref for InstanceRef<'a, C, T, I>
    where C: FlyWeightContainerTrait<T, I>,
          T: TemplateTrait<T, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        self.inst
    }
}

impl<'a, C, T, I> InstanceRef<'a, C, T, I>
    where C: FlyWeightContainerTrait<T, I>,
          T: TemplateTrait<T, I>,
          I: InstanceTrait<T, I> {
    /// Get reference to the container struct where this instance lives in.
    pub fn container(&self) -> &C {
        self.container
    }

    /// Get a reference to the parent of this instance.
    pub fn parent(&self) -> TemplateRef<C, T, I> {
        let parent = &self.container.fwc().templates[&self.inst().parent_id];
        TemplateRef::new(
            self.container(),
            parent,
        )
    }

    /// Get a reference to the template of this instance.
    pub fn template(&self) -> TemplateRef<C, T, I> {
        let template = &self.container.fwc().templates[&self.inst().template_id];
        TemplateRef::new(
            self.container(),
            template,
        )
    }
}


#[test]
fn test() {
    #[derive(Default)]
    struct Netlist {
        container: FlyWeightContainer<Circuit, CircuitInstance>
    }

    impl Netlist {
        impl_flyweight_container! {Circuit, CircuitInstance}
    }

    impl FlyWeightContainerTrait<Circuit, CircuitInstance> for Netlist {
        fn fwc(&self) -> &FlyWeightContainer<Circuit, CircuitInstance> {
            &self.container
        }

        fn fwc_mut(&mut self) -> &mut FlyWeightContainer<Circuit, CircuitInstance> {
            &mut self.container
        }
    }

    struct Circuit {
        tpl: Template<Circuit, CircuitInstance>
    }

    impl Circuit {
        impl_template! {Circuit, CircuitInstance}
    }

    impl TemplateTrait<Circuit, CircuitInstance> for Circuit {
        fn tpl(&self) -> &Template<Circuit, CircuitInstance> {
            &self.tpl
        }

        fn tpl_mut(&mut self) -> &mut Template<Circuit, CircuitInstance> {
            &mut self.tpl
        }

        fn new(t: Template<Circuit, CircuitInstance>) -> Self {
            Circuit { tpl: t }
        }
    }

    struct CircuitInstance {
        inst: Instance<Circuit, CircuitInstance>
    }

    impl CircuitInstance {
        impl_instance! {Circuit, CircuitInstance}
    }

    impl InstanceTrait<Circuit, CircuitInstance> for CircuitInstance {
        fn inst(&self) -> &Instance<Circuit, CircuitInstance> {
            &self.inst
        }

        fn inst_mut(&mut self) -> &mut Instance<Circuit, CircuitInstance> {
            &mut self.inst
        }

        fn new(i: Instance<Circuit, CircuitInstance>) -> Self {
            CircuitInstance { inst: i }
        }
    }

    let mut netlist = Netlist::default();

    let id_a = netlist.create_template("A".into());
    let id_b = netlist.create_template("B".into());
    assert_eq!(netlist.each_template().len(), 2);

    let _inst1 = netlist.create_instance(&id_a, &id_b, "instB".into());

    let a = netlist.template_by_id(&id_a);
    let b = netlist.template_by_id(&id_b);

    assert_eq!(a.each_instance_ref().len(), 1);
    assert_eq!(b.each_instance_ref().len(), 0);

    assert_eq!(a.each_dependency_id().len(), 1);
    assert_eq!(a.each_dependent_template_id().len(), 0);

    assert_eq!(b.each_dependency_id().len(), 0);
    assert_eq!(b.each_dependent_template_id().len(), 1);

    let inst_b = a.instance_ref_by_name("instB").unwrap();
    assert_eq!(inst_b.template().id(), id_b);
    assert_eq!(inst_b.parent().id(), id_a);
    assert_eq!(inst_b.parent_id(), id_a);
}
