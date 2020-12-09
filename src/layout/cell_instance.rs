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
use crate::prelude::*;

use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct CellInstance<C: CoordinateType> {
    /// Reference to the cell of which this is an instance.
    cell: Rc<Cell<C>>,
    /// Transformation to put the cell to the right place an into the right scale/rotation.
    transform: SimpleTransform<C>
    // TODO: Repetition
}

impl<C: CoordinateType> CellInstance<C> {
    /// Create a new cell instance.
    pub fn new(cell_ref: Rc<Cell<C>>, transform: SimpleTransform<C>) -> Self {
        CellInstance {
            cell: cell_ref,
            transform,
        }
    }

    /// Get reference to the instantiated cell.
    pub fn cell(&self) -> Rc<Cell<C>> {
        self.cell.clone()
    }

    /// Get the transformation describing the location, orientation and magnification of this cell instance.
    pub fn get_transform(&self) -> SimpleTransform<C> {
        self.transform.clone()
    }
}