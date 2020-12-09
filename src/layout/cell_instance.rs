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