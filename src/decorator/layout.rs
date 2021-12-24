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

use crate::traits::{HierarchyBase, LayoutBase, LayoutEdit, HierarchyEdit};
use crate::prelude::{LayerInfo, Rect, SimpleTransform, Geometry, PropertyValue, UInt};

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

    fn dbu(&self) -> <Self as LayoutBase>::Coord {
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

    fn bounding_box_per_layer(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Option<Rect<<Self as LayoutBase>::Coord>> {
        self.base().bounding_box_per_layer(cell, layer)
    }

    fn bounding_box(&self, cell: &Self::CellId) -> Option<Rect<<Self as LayoutBase>::Coord>> {
        self.base().bounding_box(cell)
    }

    fn each_shape_id(&self, cell: &Self::CellId, layer: &Self::LayerId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.base().each_shape_id(cell, layer)
    }

    fn for_each_shape<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(&Self::ShapeId, &Geometry<<Self as LayoutBase>::Coord>) -> () {
        self.base().for_each_shape(cell, layer, f)
    }

    fn with_shape<F, R>(&self, shape_id: &Self::ShapeId, f: F) -> R
        where F: FnMut(&Self::LayerId, &Geometry<<Self as LayoutBase>::Coord>) -> R {
        self.base().with_shape(shape_id, f)
    }

    fn parent_of_shape(&self, shape_id: &Self::ShapeId) -> (Self::CellId, Self::LayerId) {
        self.base().parent_of_shape(shape_id)
    }

    fn for_each_shape_recursive<F>(&self, cell: &Self::CellId, layer: &Self::LayerId, f: F)
        where F: FnMut(SimpleTransform<<Self as LayoutBase>::Coord>, &Self::ShapeId, &Geometry<<Self as LayoutBase>::Coord>) -> () {
        self.base().for_each_shape_recursive(cell, layer, f)
    }

    fn get_transform(&self, cell_inst: &Self::CellInstId) -> SimpleTransform<<Self as LayoutBase>::Coord> {
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

    struct DummyDecorator<T>(T);

    impl<'a, H: HierarchyBase> HierarchyBaseDecorator for DummyDecorator<&'a H> {
        type H = H;

        fn base(&self) -> &Self::H {
            self.0
        }
    }

    impl<'a, H: LayoutBase> LayoutBaseDecorator for DummyDecorator<&'a H> {
        type L = H;

        fn base(&self) -> &Self::L {
            self.0
        }
    }

    assert_eq!(chip.each_layer().count(), 1);
    let decorated_chip = DummyDecorator(&chip);
    assert_eq!(decorated_chip.each_layer().count(), 1);
}

/// Define the same functions as [`LayoutEdit`] but just prepend a `d_` to
/// avoid naming conflicts.
/// The default implementation just forwards the call to the `base()`.
/// This allows to selectively re-implement some functions or fully delegate
/// the trait to an attribute of a struct.
pub trait LayoutEditDecorator
    where Self: Sized
{
    type L: LayoutEdit;

    /// Get a mutable reference to the underlying data structure.
    fn mut_base(&mut self) -> &mut Self::L;

    fn d_set_dbu(&mut self, dbu: <Self::L as LayoutBase>::Coord) {
        self.mut_base().set_dbu(dbu)
    }

    fn d_create_layer(&mut self, index: UInt, datatype: UInt) -> <Self::L as LayoutBase>::LayerId {
        self.mut_base().create_layer(index, datatype)
    }

    fn d_set_layer_name(&mut self, layer: &<Self::L as LayoutBase>::LayerId, name: Option<<Self::L as HierarchyBase>::NameType>) -> Option<<Self::L as HierarchyBase>::NameType> {
        self.mut_base().set_layer_name(layer, name)
    }

    fn d_insert_shape(&mut self, parent_cell: &<Self::L as HierarchyBase>::CellId, layer: &<Self::L as LayoutBase>::LayerId, geometry: Geometry<<Self::L as LayoutBase>::Coord>) -> <Self::L as LayoutBase>::ShapeId {
        self.mut_base().insert_shape(parent_cell, layer, geometry)
    }

    fn d_remove_shape(&mut self, shape_id: &<Self::L as LayoutBase>::ShapeId) -> Option<Geometry<<Self::L as LayoutBase>::Coord>> {
        self.mut_base().remove_shape(shape_id)
    }

    fn d_replace_shape(&mut self, shape_id: &<Self::L as LayoutBase>::ShapeId, geometry: Geometry<<Self::L as LayoutBase>::Coord>) -> Geometry<<Self::L as LayoutBase>::Coord> {
        self.mut_base().replace_shape(shape_id, geometry)
    }

    fn d_set_transform(&mut self, cell_inst: &<Self::L as HierarchyBase>::CellInstId, tf: SimpleTransform<<Self::L as LayoutBase>::Coord>) {
        self.mut_base().set_transform(cell_inst, tf)
    }

    fn d_set_shape_property(&mut self, shape: &<Self::L as LayoutBase>::ShapeId, key: <Self::L as HierarchyBase>::NameType, value: PropertyValue) {
        self.mut_base().set_shape_property(shape, key, value)
    }
}

impl<T, L> LayoutEdit for T
    where
        T: LayoutBase<Coord=L::Coord, ShapeId=L::ShapeId, LayerId=L::LayerId>
        + HierarchyEdit<NameType=L::NameType, CellId=L::CellId, CellInstId=L::CellInstId>
        + LayoutEditDecorator<L=L>,
        L: LayoutEdit + 'static
{
    fn set_dbu(&mut self, dbu: Self::Coord) {
        self.d_set_dbu(dbu)
    }

    fn create_layer(&mut self, index: UInt, datatype: UInt) -> Self::LayerId {
        self.d_create_layer(index, datatype)
    }

    fn set_layer_name(&mut self, layer: &Self::LayerId, name: Option<Self::NameType>) -> Option<Self::NameType> {
        self.d_set_layer_name(layer, name)
    }

    fn insert_shape(&mut self, parent_cell: &Self::CellId, layer: &Self::LayerId, geometry: Geometry<Self::Coord>) -> Self::ShapeId {
        self.d_insert_shape(parent_cell, layer, geometry)
    }

    fn remove_shape(&mut self, shape_id: &Self::ShapeId) -> Option<Geometry<Self::Coord>> {
        self.d_remove_shape(shape_id)
    }

    fn replace_shape(&mut self, shape_id: &Self::ShapeId, geometry: Geometry<Self::Coord>) -> Geometry<Self::Coord> {
        self.d_replace_shape(shape_id, geometry)
    }

    fn set_transform(&mut self, cell_inst: &Self::CellInstId, tf: SimpleTransform<Self::Coord>) {
        self.d_set_transform(cell_inst, tf)
    }

    fn set_shape_property(&mut self, shape: &Self::ShapeId, key: Self::NameType, value: PropertyValue) {
        self.d_set_shape_property(shape, key, value)
    }
}


#[test]
fn test_layout_edit_decorator() {
    use crate::chip::Chip;
    use super::hierarchy::{HierarchyBaseDecorator, HierarchyEditDecorator};
    use crate::prelude::*;

    let mut chip = Chip::new();
    chip.create_layer(0, 0);

    struct DummyDecorator<T>(T);

    impl<'a, H: HierarchyBase> HierarchyBaseDecorator for DummyDecorator<&'a mut H> {
        type H = H;

        fn base(&self) -> &Self::H {
            self.0
        }
    }

    impl<'a, H: HierarchyEdit> HierarchyEditDecorator for DummyDecorator<&'a mut H> {
        type H = H;

        fn mut_base(&mut self) -> &mut Self::H {
            self.0
        }

        fn d_new() -> Self {
            unimplemented!()
        }
    }

    impl<'a, H: LayoutBase> LayoutBaseDecorator for DummyDecorator<&'a mut H> {
        type L = H;

        fn base(&self) -> &Self::L {
            self.0
        }
    }

    impl<'a, L: LayoutEdit> LayoutEditDecorator for DummyDecorator<&'a mut L> {
        type L = L;

        fn mut_base(&mut self) -> &mut Self::L {
            self.0
        }
    }

    let mut decorated_chip = DummyDecorator(&mut chip);
    decorated_chip.create_layer(0, 0);
}