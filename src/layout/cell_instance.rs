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

//! A cell instance is the effective usage of a cell.

use crate::prelude::*;

use std::rc::Weak;
use std::hash::{Hash, Hasher};
use crate::property_storage::{WithProperties, PropertyStore};
use std::cell::RefCell;

/// An actual instance of a cell.
#[derive(Clone, Debug)]
pub struct CellInstance<C: CoordinateType> {
    /// ID of the parent cell.
    pub(super) parent_cell_id: CellIndex,
    /// Identifier. Uniquely identifies the instance within the parent cell.
    pub(super) id: CellInstId,
    /// Reference to the cell of which this is an instance.
    pub(super) cell: Weak<Cell<C>>,
    /// Cell where this instance lives in.
    pub(super) parent_cell: Weak<Cell<C>>,
    /// Transformation to put the cell to the right place an into the right scale/rotation.
    pub(super) transform: RefCell<SimpleTransform<C>>,
    // TODO: Repetition
}

impl<C: CoordinateType> Eq for CellInstance<C> {}

impl<C: CoordinateType> PartialEq for CellInstance<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.parent_cell_id == other.parent_cell_id
    }
}

impl<C: CoordinateType> Hash for CellInstance<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.parent_cell_id.hash(state);
    }
}

impl<C: CoordinateType> CellInstance<C> {
    /// Get the ID of this cell instance.
    pub fn id(&self) -> CellInstId {
        self.id
    }

    /// Get reference to the template cell.
    pub fn cell(&self) -> Weak<Cell<C>> {
        self.cell.clone()
    }


    /// Get ID of the template cell.
    pub fn cell_id(&self) -> CellIndex {
        // TODO: Include the ID in the struct?
        self.cell.upgrade().unwrap().index()
    }

    /// Get reference to the cell where this instance lives in.
    pub fn parent_cell(&self) -> Weak<Cell<C>> {
        self.parent_cell.clone()
    }

    /// Get the transformation describing the location, orientation and magnification of this cell instance.
    pub fn get_transform(&self) -> SimpleTransform<C> {
        self.transform.borrow().clone()
    }

    /// Set the transformation describing the location, orientation and magnification of this cell instance.
    pub fn set_transform(&self, tf: SimpleTransform<C>) {
        self.transform.replace(tf);
    }
}


impl<C: CoordinateType> WithProperties for CellInstance<C> {
    type Key = String;

    fn with_properties<F, R>(&self, f: F) -> R
        where F: FnOnce(Option<&PropertyStore<Self::Key>>) -> R {
        f(
            // Get the property store from the parent cell.
            self.parent_cell()
                .upgrade()
                .unwrap()
                .instance_properties.borrow()
                .get(&self.id())
        )
    }

    fn with_properties_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut PropertyStore<Self::Key>) -> R {
        f(
            // Get the property store from the parent cell.
            self.parent_cell()
                .upgrade()
                .unwrap()
                .instance_properties.borrow_mut()
                .entry(self.id())
                .or_insert(PropertyStore::default())
        )
    }
}
