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

//! Traits for layout data types.


use std::borrow::Borrow;
use std::hash::Hash;

/// Most basic trait of a layout.
pub trait LayoutBase {
    /// Type for names of circuits, instances, pins, etc.
    type NameType: Eq + Hash + From<String> + Clone + Borrow<String> + Borrow<str>;
    /// Layer identifier type.
    type LayerId: Eq + Hash + Clone;
    /// Cell/module identifier type.
    type CellId: Eq + Hash + Clone;
    /// Cell instance identifier type.
    type CellInstId: Eq + Hash + Clone;


    /// Create a new empty netlist.
    fn new() -> Self;

    /// Find a cell by its name.
    /// Return the cell with the given name. Returns `None` if the cell does not exist.
    fn cell_by_name<N: ?Sized + Eq + Hash>(&self, name: &N) -> Option<Self::CellId>
        where Self::NameType: Borrow<N>;

    /// Iterate over all cells.
    fn each_cell<'a>(&'a self) -> Box<dyn Iterator<Item=&Self::CellId> + 'a>;

}


/// Trait for layouts that support editing.
pub trait LayoutEdit
    where Self: LayoutBase {

    /// Create a new and empty cell.
    fn create_cell(&mut self, name: Self::NameType) -> Self::CellId;

    /// Delete the given cell if it exists.
    fn remove_cell(&mut self, cell_id: &Self::CellId);

    /// Create a new cell instance.
    fn create_cell_instance(&mut self,
                               parent_cell: &Self::CellId,
                               template_cell: &Self::CellId,
                               name: Option<Self::NameType>) -> Self::CellInstId;

    /// Remove cell instance if it exists.
    fn remove_cell_instance(&mut self, id: &Self::CellInstId);

}