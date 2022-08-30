// SPDX-FileCopyrightText: 2018-2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Profile data-base performance by measuring time spent in function calls.

use crate::prelude::*;
use crate::decorator::{Decorator, MutDecorator};
use crate::decorator::hierarchy::*;
use crate::decorator::layout::*;
use crate::decorator::netlist::*;

use std::time::{Duration, Instant};



/// Wrapper around netlist, layout and L2N structures that allows measuring time spend in function calls.
///
/// # Types
/// * `T`: Underlying data structure.
pub struct DBPerf<'a, T> {
    /// Underlying data structure.
    chip: &'a mut T,
    perf_data: PerfCounters,
}

impl<'a, T> DBPerf<'a, T> {

    /// Wrap the `chip` structure into a performance counter.
    pub fn new(chip: &'a mut T) -> Self {
        Self {
            chip,
            perf_data: Default::default()
        }
    }

    /// Access the performance counters.
    pub fn perf_data(&self) -> &PerfCounters {
        &self.perf_data
    }
}

/// Statistics on function calls of the DB.
#[derive(Debug, Default, Clone)]
pub struct PerfCounters {
    /// Performance counters of `insert_shape()`.
    pub insert_shape: PerfCounter,
}

/// Statistics on calls of a single function.
#[derive(Debug, Default, Clone)]
pub struct PerfCounter {
    /// Number of function calls.
    pub num_calls: usize,
    /// Total time spent in this function.
    pub duration: Duration,
}

impl PerfCounter {
    fn register_call(&mut self) {
        self.num_calls += 1;
    }
}

impl<'a, T> Decorator for DBPerf<'a, T> {
    type D = T;

    fn base(&self) -> &Self::D {
        &self.chip
    }
}

impl<'a, T> MutDecorator for DBPerf<'a, T> {
    fn mut_base(&mut self) -> &mut Self::D {
        &mut self.chip
    }
}


// Inherit everything from HierarchyBase.
impl<'a, H: HierarchyBase + 'static> HierarchyBaseDecorator for DBPerf<'a, H> {
    type NameType = H::NameType;
    type CellId = H::CellId;
    type CellInstId = H::CellInstId;
}

// Inherit everything from LayoutBase.
impl<'a, L: LayoutBase + 'static> LayoutBaseDecorator for DBPerf<'a, L> {}


// Inherit everything from NetlistBase.
impl<'a, N: NetlistBase + 'static> NetlistBaseDecorator for DBPerf<'a, N> {}

// Inherit everything from HierarchyEdit.
impl<'a, H: HierarchyEdit + 'static> HierarchyEditDecorator for DBPerf<'a, H> {

    fn d_new() -> Self {
        unimplemented!()
    }

}

// Inherit everything from LayoutEdit.
impl<'a, L: LayoutEdit + 'static> LayoutEditDecorator for DBPerf<'a, L> {

    fn d_insert_shape(&mut self, parent_cell: &<Self::D as HierarchyBase>::CellId, layer: &<Self::D as LayoutBase>::LayerId, geometry: Geometry<<Self::D as LayoutBase>::Coord>) -> <Self::D as LayoutBase>::ShapeId {
        let counter = &mut self.perf_data.insert_shape;
        counter.register_call();
        let t = Instant::now();
        let ret = self.chip.insert_shape(parent_cell, layer, geometry);
        counter.duration += t.elapsed();
        ret
    }

}


// Inherit everything from NetlistBase.
impl<'a, N: NetlistEdit + 'static> NetlistEditDecorator for DBPerf<'a, N> {}