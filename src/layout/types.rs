
use super::index::{Index, IndexGenerator};
use super::cell::Cell;
use super::cell_instance::CellInstance;
use super::layout::LayerInfo;

pub type UInt = u32;
pub type SInt = i32;

/// Integer coordinate type.
pub type Coord = i32;

/// Data type used for identifying a layer.
pub type LayerIndex = Index<LayerInfo>;
pub type LayerIndexGenerator = IndexGenerator<LayerInfo>;

/// Data type used for identifying a cell.
pub type CellIndex = Index<Cell<Coord>>;
pub type CellIndexGenerator = IndexGenerator<Cell<Coord>>;

/// Data type used for identifying a cell instance.
pub type CellInstId = Index<CellInstance<Coord>>;
pub type CellInstIndexdGenerator = IndexGenerator<CellInstance<Coord>>;

