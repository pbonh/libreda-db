// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! # Experimental
//!
//! Simplify usage of cell libraries.

// TODO: Remove when implemented.
#![allow(unused)]

use crate::prelude::HierarchyBase;

/// Identifier of a library.
#[derive(Copy, Clone, Debug, Hash, PartialOrd, PartialEq, Eq, Ord)]
pub struct LibraryId(usize);

pub struct LibraryWrapper<'a, T> {
    libraries: Vec<&'a T>,
    owned: T,
}

impl<'a, T> LibraryWrapper<'a, T> {
    /// Register a library.
    pub fn add_library(&mut self, library: &'a T) -> LibraryId {
        self.libraries.push(library);
        LibraryId(self.libraries.len())
    }

    fn get_library(&self, library_id: &LibraryId) -> &T {
        if library_id == &LibraryId(0) {
            &self.owned
        } else {
            &self.libraries[library_id.0]
        }
    }

    /// Get reference to owned data and to libraries.
    fn libraries(&self) -> impl Iterator<Item = (LibraryId, &T)> {
        std::iter::once((LibraryId(0), &self.owned)).chain(
            self.libraries
                .iter()
                .enumerate()
                .map(|(id, lib)| (LibraryId(id + 1), *lib)),
        )
    }
}

impl<'a, T> HierarchyBase for LibraryWrapper<'a, T>
where
    T: HierarchyBase,
{
    type NameType = T::NameType;
    type CellId = (LibraryId, T::CellId);
    type CellInstId = (LibraryId, T::CellInstId);

    /// Find a cell by name.
    /// Precedence is: Current data base, then libraries in the order where they where added.
    fn cell_by_name(&self, name: &str) -> Option<Self::CellId> {
        // Find in libraries.
        self.libraries()
            .flat_map(|(lib_id, lib)| lib.cell_by_name(name).map(|cell| (lib_id, cell)))
            .next()
    }

    fn cell_instance_by_name(
        &self,
        (lib_id, parent_cell): &Self::CellId,
        name: &str,
    ) -> Option<Self::CellInstId> {
        self.get_library(lib_id)
            .cell_instance_by_name(parent_cell, name)
            .map(|inst| (*lib_id, inst))
    }

    fn cell_name(&self, (lib_id, cell): &Self::CellId) -> Self::NameType {
        self.get_library(lib_id).cell_name(cell)
    }

    fn cell_instance_name(&self, (lib_id, cell_inst): &Self::CellInstId) -> Option<Self::NameType> {
        self.get_library(lib_id).cell_instance_name(cell_inst)
    }

    fn parent_cell(&self, (lib_id, cell_instance): &Self::CellInstId) -> Self::CellId {
        // self.get_library(lib_id)
        //     .parent_cell(cell_instance)
        unimplemented!()
    }

    fn template_cell(&self, (lib_id, cell_instance): &Self::CellInstId) -> Self::CellId {
        // self.get_library(lib_id)
        //     .template_cell(cell_instance)
        unimplemented!()
    }

    fn for_each_cell<F>(&self, f: F)
    where
        F: FnMut(Self::CellId) -> (),
    {
        unimplemented!()
    }

    fn for_each_cell_instance<F>(&self, cell: &Self::CellId, f: F)
    where
        F: FnMut(Self::CellInstId) -> (),
    {
        unimplemented!()
    }

    fn for_each_cell_dependency<F>(&self, cell: &Self::CellId, f: F)
    where
        F: FnMut(Self::CellId) -> (),
    {
        unimplemented!()
    }

    fn for_each_dependent_cell<F>(&self, cell: &Self::CellId, f: F)
    where
        F: FnMut(Self::CellId) -> (),
    {
        unimplemented!()
    }

    fn for_each_cell_reference<F>(&self, cell: &Self::CellId, f: F)
    where
        F: FnMut(Self::CellInstId) -> (),
    {
        unimplemented!()
    }

    fn num_child_instances(&self, cell: &Self::CellId) -> usize {
        unimplemented!()
    }

    fn num_cells(&self) -> usize {
        unimplemented!()
    }
}
