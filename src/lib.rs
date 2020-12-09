//! `iron-db` is a database for VLSI physical design. The core components are data structures for efficient
//! representation of geometries and circuit netlists for chip layouts.

pub extern crate iron_shapes;
extern crate itertools;
extern crate genawaiter;

// Public modules.
pub mod prelude;
pub mod netlist;
pub mod layout;

// Private modules.
// mod refset; // Not currently used.
// mod ref_wrapper; // Not currently used.
