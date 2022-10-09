// SPDX-FileCopyrightText: 2018-2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Add fast region queries to layouts.

// TODO: Remove this, once implemented.
#![allow(unused_variables)]

use fnv::FnvHashMap;
use num_traits::{PrimInt, Signed};
use rstar::{RTree, RTreeObject};

use crate::decorator::hierarchy::*;
use crate::decorator::l2n::*;
use crate::decorator::layout::*;
use crate::decorator::netlist::*;
use crate::decorator::{Decorator, MutDecorator};
use crate::prelude::*;

/// Wrapper around netlist, layout and L2N structures that allows fast region queries.
///
/// # Types
/// * `T`: Underlying data structure.
pub struct RegionSearchAdapter<'a, T>
where
    T: LayoutBase,
    T::Coord: PrimInt + Signed + std::fmt::Debug,
{
    /// Underlying data structure.
    chip: &'a mut T,
    /// RTrees containing geometric shapes.
    shape_rtrees:
        FnvHashMap<T::CellId, FnvHashMap<T::LayerId, RTree<ShapeEntry<T::ShapeId, T::Coord>>>>,
    /// RTree containing child instances.
    instance_rtree: FnvHashMap<T::CellId, RTree<CellInstanceEntry<T>>>,
    /// Cache for bounding boxes of cells.
    cell_bounding_boxes: FnvHashMap<T::CellId, Rect<T::Coord>>,
}

/// Wrapper for shapes.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeEntry<ShapeId, Coord> {
    bounding_box: Rect<Coord>,
    shape_id: ShapeId,
}

/// Wrapper for cell instances
#[derive(Debug, Clone, PartialEq)]
pub struct CellInstanceEntry<L: LayoutBase> {
    bounding_box: Rect<L::Coord>,
    cell_inst_id: L::CellInstId,
}

// Make `ShapeEntry` usable within RTrees.
impl<ShapeId, Coord> BoundingBox<Coord> for ShapeEntry<ShapeId, Coord>
where
    Coord: PrimInt,
{
    fn bounding_box(&self) -> Rect<Coord> {
        self.bounding_box
    }
}

// Make `ShapeEntry` usable within RTrees.
impl<ShapeId, Coord> RTreeObject for ShapeEntry<ShapeId, Coord>
where
    Coord: PrimInt + Signed + std::fmt::Debug,
{
    type Envelope = rstar::AABB<[Coord; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let bbox = self.bounding_box();

        rstar::AABB::from_corners(bbox.lower_left().into(), bbox.upper_right().into())
    }
}

// Make `CellInstanceEntry` usable within RTrees.
impl<L> BoundingBox<L::Coord> for CellInstanceEntry<L>
where
    L: LayoutBase,
    L::Coord: PrimInt,
{
    fn bounding_box(&self) -> Rect<L::Coord> {
        self.bounding_box
    }
}

// Make `CellInstanceEntry` usable within RTrees.
impl<L> RTreeObject for CellInstanceEntry<L>
where
    L: LayoutBase,
    L::Coord: PrimInt + Signed + std::fmt::Debug,
{
    type Envelope = rstar::AABB<[L::Coord; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let bbox = self.bounding_box();

        rstar::AABB::from_corners(bbox.lower_left().into(), bbox.upper_right().into())
    }
}

impl<'a, T> RegionSearchAdapter<'a, T>
where
    T: LayoutBase,
    T::Coord: PrimInt + Signed + std::fmt::Debug,
{
    /// Add fast region query capability to `chip`.
    pub fn new(chip: &'a mut T) -> Self {
        let mut region_search = Self {
            chip,
            shape_rtrees: Default::default(),
            instance_rtree: Default::default(),
            cell_bounding_boxes: Default::default(),
        };

        region_search.create_shape_trees();
        region_search.create_cell_instance_trees();

        region_search
    }

    fn create_shape_trees(&mut self) {
        // Process cells starting with leaf-cells to top cells.
        let mut cell_rtrees: FnvHashMap<_, _> = Default::default();
        for cell in self.chip.each_cell_bottom_to_top() {
            let mut rtrees: FnvHashMap<_, RTree<_>> = Default::default();

            for layer in self.chip.each_layer() {
                // Collect all shapes which have a bounding box.
                let mut all_shapes = vec![];
                self.chip
                    .for_each_shape(&cell, &layer, |shape_id, geometry| {
                        if let Some(bounding_box) = geometry.try_bounding_box() {
                            let rtree_entry = ShapeEntry {
                                bounding_box,
                                shape_id: shape_id.clone(),
                            };
                            all_shapes.push(rtree_entry)
                        }
                    });

                // Create RTree.
                let rtree = RTree::bulk_load(all_shapes);

                rtrees.insert(layer, rtree);
            }
            cell_rtrees.insert(cell, rtrees);
        }

        self.shape_rtrees = cell_rtrees;
    }

    fn create_cell_instance_trees(&mut self) {
        for cell in self.chip.each_cell_bottom_to_top() {
            // Compute bounding box of the shapes (without subcells).
            let shape_bbox = self.compute_shape_bbox(&cell);

            // Register all instances in the rtree (their bounding boxes are known by now).
            let instance_entries: Vec<_> = self
                .chip
                .each_cell_instance(&cell)
                .filter_map(|inst| {
                    self.cell_bounding_boxes
                        .get(&self.chip.template_cell(&inst))
                        .map(|bbox| (inst, bbox))
                })
                .map(|(inst, bbox)| {
                    let tf = self.chip.get_transform(&inst);
                    let bbox_transformed = bbox.transform(|p| tf.transform_point(p));

                    CellInstanceEntry {
                        bounding_box: bbox_transformed,
                        cell_inst_id: inst,
                    }
                })
                .collect();

            let rtree = RTree::bulk_load(instance_entries);

            let cell_inst_bbox = if rtree.size() > 0 {
                let envelope = rtree.root().envelope();
                let bbox = Rect::new(envelope.lower(), envelope.upper());
                Some(bbox)
            } else {
                None
            };

            // Combine both bounding boxes.
            let bbox = cell_inst_bbox
                .into_iter()
                .chain(shape_bbox)
                .reduce(|acc, b| acc.add_rect(&b));

            if let Some(bbox) = bbox {
                self.cell_bounding_boxes.insert(cell.clone(), bbox);
            }

            self.instance_rtree.insert(cell, rtree);
        }
    }

    /// Compute the bounding box of all shapes in a cell, excluding sub cells.
    fn compute_shape_bbox(&self, cell_id: &T::CellId) -> Option<Rect<T::Coord>> {
        self.shape_rtrees
            .get(cell_id)
            .into_iter()
            // Get rtree of each layer.
            .flat_map(|layer_trees| layer_trees.values())
            // Get the bounding box of the layer.
            .map(|rtree| rtree.root().envelope())
            // Convert to Rect
            .map(|envelope| Rect::new(envelope.lower(), envelope.upper()))
            // Reduce to a single bounding box.
            .reduce(|acc, b| acc.add_rect(&b))
    }

    /// Compute bounding box of all subcells, excluding shapes in the current cell.
    fn compute_subcell_bboxes(&self, cell_id: &T::CellId) -> Option<Rect<T::Coord>> {
        if let Some(instance_rtree) = self.instance_rtree.get(cell_id) {
            if instance_rtree.size() > 0 {
                let envelope = instance_rtree.root().envelope();
                Some(Rect::new(envelope.lower(), envelope.upper()))
            } else {
                None
            }
        } else {
            None
        }
        // let subcell_bboxes = self.chip.each_cell_instance(cell_id)
        //     .filter_map(|inst| {
        //         self.cell_bounding_boxes.get(&self.chip.template_cell(&inst))
        //             .map(|bbox| (inst, bbox))
        //     })
        //     .map(|(inst, bbox)| {
        //         let tf = self.chip.get_transform(&inst);
        //         let bbox_transformed = bbox.transform(|p| tf.transform_point(p));
        //         bbox_transformed
        //     });
        //
        // subcell_bboxes.reduce(|acc, b| {
        //     acc.add_rect(&b)
        // })
    }
}

impl<'a, T> Decorator for RegionSearchAdapter<'a, T>
where
    T: LayoutBase,
    T::Coord: PrimInt + Signed + std::fmt::Debug,
{
    type D = T;

    fn base(&self) -> &Self::D {
        &self.chip
    }
}

impl<'a, T> MutDecorator for RegionSearchAdapter<'a, T>
where
    T: LayoutBase,
    T::Coord: PrimInt + Signed + std::fmt::Debug,
{
    fn mut_base(&mut self) -> &mut Self::D {
        &mut self.chip
    }
}

// Inherit everything from HierarchyBase.
impl<'a, H> HierarchyBaseDecorator for RegionSearchAdapter<'a, H>
where
    H: LayoutBase + 'static,
    H::Coord: PrimInt + Signed + std::fmt::Debug,
{
    type NameType = H::NameType;
    type CellId = H::CellId;
    type CellInstId = H::CellInstId;
}

//
// // Inherit everything from LayoutBase.
impl<'a, L> LayoutBaseDecorator for RegionSearchAdapter<'a, L>
where
    L: LayoutBase + 'static,
    L::Coord: PrimInt + Signed + std::fmt::Debug,
{
}

impl<'a, L> RegionSearch for RegionSearchAdapter<'a, L>
where
    L: LayoutBase + 'static,
    L::Coord: PrimInt + Signed + std::fmt::Debug,
{
    fn each_shape_in_region_per_layer(
        &self,
        cell: &Self::CellId,
        layer_id: &Self::LayerId,
        search_region: &Rect<Self::Coord>,
    ) -> Box<dyn Iterator<Item = Self::ShapeId> + '_> {
        let aabb = rect2aabb(search_region);

        let intersecting_instances = self
            .shape_rtrees
            .get(cell)
            .expect("cell not found")
            .get(layer_id)
            .into_iter()
            .flat_map(move |rtree| rtree.locate_in_envelope_intersecting(&aabb))
            .map(|shape_entry| shape_entry.shape_id.clone());

        Box::new(intersecting_instances)
    }

    fn each_cell_instance_in_region(
        &self,
        cell: &Self::CellId,
        search_region: &Rect<Self::Coord>,
    ) -> Box<dyn Iterator<Item = Self::CellInstId> + '_> {
        let intersecting_instances = self
            .instance_rtree
            .get(cell)
            .expect("cell not found")
            .locate_in_envelope_intersecting(&rect2aabb(search_region))
            .map(|instance_entry| instance_entry.cell_inst_id.clone());

        Box::new(intersecting_instances)
    }
}

/// Convert a rectangle into an axis aligned bounding box used by RStar.
fn rect2aabb<Crd>(r: &Rect<Crd>) -> rstar::AABB<[Crd; 2]>
where
    Crd: PrimInt + Signed + std::fmt::Debug,
{
    rstar::AABB::from_corners(r.lower_left().into(), r.upper_right().into())
}

// Inherit everything from NetlistBase.
impl<'a, N> NetlistBaseDecorator for RegionSearchAdapter<'a, N>
where
    N: LayoutBase + NetlistBase + 'static,
    N::Coord: PrimInt + Signed + std::fmt::Debug,
{
}

impl<'a, LN> L2NBaseDecorator for RegionSearchAdapter<'a, LN>
where
    LN: L2NBase + 'static,
    LN::Coord: PrimInt + Signed + std::fmt::Debug,
{
}

// Inherit everything from HierarchyEdit.
impl<'a, H> HierarchyEditDecorator for RegionSearchAdapter<'a, H>
where
    H: HierarchyEdit + LayoutBase + 'static,
    H::Coord: PrimInt + Signed + std::fmt::Debug,
{
    fn d_new() -> Self {
        unimplemented!()
    }

    fn d_create_cell(&mut self, name: H::NameType) -> H::CellId {
        todo!()
    }

    fn d_remove_cell(&mut self, cell_id: &H::CellId) {
        todo!()
    }

    fn d_create_cell_instance(
        &mut self,
        parent_cell: &H::CellId,
        template_cell: &H::CellId,
        name: Option<H::NameType>,
    ) -> H::CellInstId {
        todo!()
    }

    fn d_remove_cell_instance(&mut self, inst: &H::CellInstId) {
        todo!()
    }
}

// Inherit everything from LayoutEdit.
impl<'a, L> LayoutEditDecorator for RegionSearchAdapter<'a, L>
where
    L: LayoutEdit + 'static,
    L::Coord: PrimInt + Signed + std::fmt::Debug,
{
    fn d_insert_shape(
        &mut self,
        parent_cell: &L::CellId,
        layer: &L::LayerId,
        geometry: Geometry<L::Coord>,
    ) -> L::ShapeId {
        todo!("update RTree");
        // let shape_id = self.chip.insert_shape(parent_cell, layer, geometry);
        // shape_id
    }

    fn d_remove_shape(&mut self, shape_id: &L::ShapeId) -> Option<Geometry<L::Coord>> {
        let _geo = self.chip.remove_shape(shape_id);
        todo!("update RTree")
    }

    fn d_replace_shape(
        &mut self,
        shape_id: &L::ShapeId,
        geometry: Geometry<L::Coord>,
    ) -> Geometry<L::Coord> {
        let _geo = self.chip.replace_shape(shape_id, geometry);
        todo!("update RTree")
    }

    fn d_set_transform(&mut self, cell_inst: &L::CellInstId, tf: SimpleTransform<L::Coord>) {
        self.chip.set_transform(cell_inst, tf);
        todo!("update RTree")
    }
}

// Inherit everything from NetlistBase.
impl<'a, N: NetlistEdit + 'static> NetlistEditDecorator for RegionSearchAdapter<'a, N>
where
    N: LayoutBase + NetlistEdit + 'static,
    N::Coord: PrimInt + Signed + std::fmt::Debug,
{
}

impl<'a, LN> L2NEditDecorator for RegionSearchAdapter<'a, LN>
where
    LN: L2NEdit + 'static,
    LN::Coord: PrimInt + Signed + std::fmt::Debug,
{
}

#[test]
fn test_create_region_search() {
    let mut chip = Chip::new();

    let top = chip.create_cell("TOP".to_string().into());
    let layer1 = chip.create_layer(1, 0);
    chip.insert_shape(&top, &layer1, Rect::new((0, 0), (10, 10)).into());

    let region_search = RegionSearchAdapter::new(&mut chip);

    assert_eq!(
        region_search.cell_bounding_boxes[&top],
        Rect::new((0, 0), (10, 10))
    );
}
