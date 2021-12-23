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

use crate::traits::{HierarchyBase, LayoutBase};
use crate::prelude::{LayerInfo, Rect, SimpleTransform, Geometry, PropertyValue};

/// Define the same functions as [`LayoutBase`] but just prepend a `d_` to
/// avoid naming conflicts.
/// The default implementation just forwards the call to the `base()`.
/// This allows to selectively re-implement some functions or fully delegate
/// the trait to an attribute of a struct.
pub trait LayoutBaseDecorator
    where Self: Sized
{
    type L: LayoutBase;

    /// Get a reference to the underlying data structure.
    fn base(&self) -> &Self::L;

    fn d_dbu(&self) -> <Self::L as LayoutBase>::Coord {
        self.base().dbu()
    }

    fn d_each_layer(&self) -> Box<dyn Iterator<Item=<Self::L as LayoutBase>::LayerId> + '_> {
        self.base().each_layer()
    }

    fn d_layer_info(&self, layer: &<Self::L as LayoutBase>::LayerId) -> LayerInfo<<Self::L as HierarchyBase>::NameType> {
        self.base().layer_info(layer)
    }

    fn d_find_layer(&self, index: u32, datatype: u32) -> Option<<Self::L as LayoutBase>::LayerId> {
        self.base().find_layer(index, datatype)
    }

    fn d_layer_by_name(&self, name: &str) -> Option<<Self::L as LayoutBase>::LayerId> {
        self.base().layer_by_name(name)
    }

    fn d_bounding_box_per_layer(&self, cell: &<Self::L as HierarchyBase>::CellId, layer: &<Self::L as LayoutBase>::LayerId) -> Option<Rect<<Self::L as LayoutBase>::Coord>> {
        self.base().bounding_box_per_layer(cell, layer)
    }

    fn d_bounding_box(&self, cell: &<Self::L as HierarchyBase>::CellId) -> Option<Rect<<Self::L as LayoutBase>::Coord>> {
        self.base().bounding_box(cell)
    }

    fn d_each_shape_id(&self, cell: &<Self::L as HierarchyBase>::CellId, layer: &<Self::L as LayoutBase>::LayerId) -> Box<dyn Iterator<Item=<Self::L as LayoutBase>::ShapeId> + '_> {
        self.base().each_shape_id(cell, layer)
    }

    fn d_for_each_shape<F>(&self, cell: &<Self::L as HierarchyBase>::CellId, layer: &<Self::L as LayoutBase>::LayerId, f: F)
        where F: FnMut(&<Self::L as LayoutBase>::ShapeId, &Geometry<<Self::L as LayoutBase>::Coord>) -> () {
        self.base().for_each_shape(cell, layer, f)
    }

    fn d_with_shape<F, R>(&self, shape_id: &<Self::L as LayoutBase>::ShapeId, f: F) -> R
        where F: FnMut(&<Self::L as LayoutBase>::LayerId, &Geometry<<Self::L as LayoutBase>::Coord>) -> R {
        self.base().with_shape(shape_id, f)
    }

    fn d_parent_of_shape(&self, shape_id: &<Self::L as LayoutBase>::ShapeId) -> (<Self::L as HierarchyBase>::CellId, <Self::L as LayoutBase>::LayerId) {
        self.base().parent_of_shape(shape_id)
    }

    fn d_for_each_shape_recursive<F>(&self, cell: &<Self::L as HierarchyBase>::CellId, layer: &<Self::L as LayoutBase>::LayerId, f: F)
        where F: FnMut(SimpleTransform<<Self::L as LayoutBase>::Coord>, &<Self::L as LayoutBase>::ShapeId, &Geometry<<Self::L as LayoutBase>::Coord>) -> () {
        self.base().for_each_shape_recursive(cell, layer, f)
    }

    fn d_get_transform(&self, cell_inst: &<Self::L as HierarchyBase>::CellInstId) -> SimpleTransform<<Self::L as LayoutBase>::Coord> {
        self.base().get_transform(cell_inst)
    }

    fn d_get_shape_property(&self, shape: &<Self::L as LayoutBase>::ShapeId, key: &<Self::L as HierarchyBase>::NameType) -> Option<PropertyValue> {
        self.base().get_shape_property(shape, key)
    }
}

impl<T, L> LayoutBase for T
    where
        T: HierarchyBase<NameType=L::NameType, CellId=L::CellId, CellInstId=L::CellInstId>
        + LayoutBaseDecorator<L=L>,
        L: LayoutBase + 'static
{
    type Coord = L::Coord;
    type LayerId = L::LayerId;
    type ShapeId = L::ShapeId;

    fn dbu(&self) -> Self::Coord {
        self.base().dbu()
    }

    fn each_layer(&self) -> Box<dyn Iterator<Item=Self::LayerId> + '_> {
        self.base().each_layer()
    }

    fn layer_info(&self, layer: &Self::LayerId) -> LayerInfo<Self::NameType> {
        self.base().layer_info(layer)
    }

    fn find_layer(&self, index: u32, datatype: u32) -> Option<Self::LayerId> {
        self.base().find_layer(index, datatype)
    }

    fn layer_by_name(&self, name: &str) -> Option<Self::LayerId> {
        self.base().layer_by_name(name)
    }

    fn bounding_box_per_layer(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Option<Rect<Self::Coord>> {
        self.base().bounding_box_per_layer(cell, layer)
    }

    fn bounding_box(&self, cell: &Self::CellId) -> Option<Rect<Self::Coord>> {
        self.base().bounding_box(cell)
    }

    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.base().each_shape_id(cell, layer)
    }

    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(&Self::ShapeId, &Geometry<Self::Coord>) -> () {
        self.base().for_each_shape(cell, layer, f)
    }

    fn with_shape<F, R>(&self, shape_id: &Self::ShapeId, f: F) -> R
        where F: FnMut(&Self::LayerId, &Geometry<Self::Coord>) -> R {
        self.base().with_shape(shape_id, f)
    }

    fn parent_of_shape(&self, shape_id: &Self::ShapeId) -> (Self::CellId, Self::LayerId) {
        self.base().parent_of_shape(shape_id)
    }

    fn for_each_shape_recursive<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(SimpleTransform<Self::Coord>, &Self::ShapeId, &Geometry<Self::Coord>) -> () {
        self.base().for_each_shape_recursive(cell, layer, f)
    }

    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<Self::Coord> {
        self.base().get_transform(cell_inst)
    }

    fn get_shape_property(&self, shape: &Self::ShapeId, key: &Self::NameType) -> Option<PropertyValue> {
        self.base().get_shape_property(shape, key)
    }
}


#[test]
fn test_layout_decorator() {
    use crate::chip::Chip;
    use super::hierarchy::HierarchyBaseDecorator;
    use crate::prelude::*;

    let mut chip = Chip::new();
    chip.create_layer(0, 0);

    // Decorator which increments the cell count by one.
    struct DummyDecorator<T>(T);

    impl<H: HierarchyBase> HierarchyBaseDecorator for DummyDecorator<H> {
        type H = H;

        fn base(&self) -> &Self::H {
            &self.0
        }
    }

    impl<H: LayoutBase> LayoutBaseDecorator for DummyDecorator<H> {
        type L = H;

        fn base(&self) -> &Self::L {
            &self.0
        }
    }

    assert_eq!(chip.each_layer().count(), 1);
    let decorated_chip = DummyDecorator(chip);
    assert_eq!(decorated_chip.each_layer().count(), 1);
}