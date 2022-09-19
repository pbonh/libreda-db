// SPDX-FileCopyrightText: 2022 Thomas Kramer
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::traits::*;
use crate::decorator::{Decorator, MutDecorator};
use super::layout::LayoutEditDecorator;

/// Define the same functions as [`L2NBase`] but just prepend a `d_` to
/// avoid naming conflicts.
/// The default implementation just forwards the call to the `base()`.
/// This allows to selectively re-implement some functions or fully delegate
/// the trait to an attribute of a struct.
pub trait L2NBaseDecorator: Decorator
    where Self::D: L2NBase
{
    fn d_shapes_of_net(&self, net_id: &<Self::D as NetlistBase>::NetId) -> Box<dyn Iterator<Item=<Self::D as LayoutBase>::ShapeId> + '_> {
        self.base().shapes_of_net(net_id)
    }

    fn d_shapes_of_pin(&self, pin_id: &<Self::D as NetlistBase>::PinId) -> Box<dyn Iterator<Item=<Self::D as LayoutBase>::ShapeId> + '_> {
        self.base().shapes_of_pin(pin_id)
    }

    fn d_get_net_of_shape(&self, shape_id: &<Self::D as LayoutBase>::ShapeId) -> Option<<Self::D as NetlistBase>::NetId> {
        self.base().get_net_of_shape(shape_id)
    }

    fn d_get_pin_of_shape(&self, shape_id: &<Self::D as LayoutBase>::ShapeId) -> Option<<Self::D as NetlistBase>::PinId> {
        self.base().get_pin_of_shape(shape_id)
    }
}

impl<T, N> L2NBase for T
    where
        T: HierarchyBase<NameType=N::NameType, CellId=N::CellId, CellInstId=N::CellInstId>
        + NetlistBase<PinId=N::PinId, NetId=N::NetId, PinInstId=N::PinInstId>
        + LayoutBase<LayerId=N::LayerId, ShapeId=N::ShapeId>
        + L2NBaseDecorator<D=N>,
        N: L2NBase + 'static
{
    fn shapes_of_net(&self, net_id: &Self::NetId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.d_shapes_of_net(net_id)
    }

    fn shapes_of_pin(&self, pin_id: &Self::PinId) -> Box<dyn Iterator<Item=Self::ShapeId> + '_> {
        self.d_shapes_of_pin(pin_id)
    }

    fn get_net_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::NetId> {
        self.d_get_net_of_shape(shape_id)
    }

    fn get_pin_of_shape(&self, shape_id: &Self::ShapeId) -> Option<Self::PinId> {
        self.d_get_pin_of_shape(shape_id)
    }
}


pub trait L2NEditDecorator: MutDecorator
    where Self::D: L2NEdit
{
    fn d_set_pin_of_shape(&mut self, shape_id: &<Self::D as LayoutBase>::ShapeId, pin: Option<<Self::D as NetlistBase>::PinId>) -> Option<<Self::D as NetlistBase>::PinId> {
        self.mut_base().set_pin_of_shape(shape_id, pin)
    }

    fn d_set_net_of_shape(&mut self, shape_id: &<Self::D as LayoutBase>::ShapeId, net: Option<<Self::D as NetlistBase>::NetId>) -> Option<<Self::D as NetlistBase>::NetId> {
        self.mut_base().set_net_of_shape(shape_id, net)
    }
}

impl<T, N> L2NEdit for T
    where
        T: HierarchyEdit<NameType=N::NameType, CellId=N::CellId, CellInstId=N::CellInstId>
        + NetlistEdit<PinId=N::PinId, NetId=N::NetId, PinInstId=N::PinInstId>
        + LayoutBase<Coord=N::Coord, LayerId=N::LayerId, ShapeId=N::ShapeId>
        + LayoutEditDecorator<D=N>
        + L2NEditDecorator<D=N>
        + L2NBaseDecorator<D=N>,
        N: L2NEdit + LayoutEdit + NetlistEdit + 'static
{
    fn set_pin_of_shape(&mut self, shape_id: &Self::ShapeId, pin: Option<Self::PinId>) -> Option<Self::PinId> {
        self.d_set_pin_of_shape(shape_id, pin)
    }

    fn set_net_of_shape(&mut self, shape_id: &Self::ShapeId, net: Option<Self::NetId>) -> Option<Self::NetId> {
        self.d_set_net_of_shape(shape_id, net)
    }
}

#[test]
fn test_l2n_edit_decorator() {
    use crate::chip::Chip;
    use super::{Decorator, MutDecorator};
    use super::hierarchy::*;
    use super::layout::*;
    use super::netlist::*;
    use super::l2n::*;
    use crate::prelude::*;

    let mut chip = Chip::new();
    chip.create_layer(0, 0);

    struct DummyDecorator<T>(T);

    impl<H> Decorator for DummyDecorator<&mut H> {
        type D = H;

        fn base(&self) -> &Self::D {
            self.0
        }
    }

    impl<H> MutDecorator for DummyDecorator<&mut H> {
        fn mut_base(&mut self) -> &mut Self::D {
            self.0
        }
    }

    impl<'a, H: HierarchyBase> HierarchyBaseDecorator for DummyDecorator<&'a mut H> {
        type NameType = H::NameType;
        type CellId = H::CellId;
        type CellInstId = H::CellInstId;
    }

    impl<'a, H: HierarchyEdit> HierarchyEditDecorator for DummyDecorator<&'a mut H> {
        fn d_new() -> Self {
            unimplemented!()
        }
    }

    impl<'a, H: LayoutBase> LayoutBaseDecorator for DummyDecorator<&'a mut H> {}

    impl<'a, L: LayoutEdit> LayoutEditDecorator for DummyDecorator<&'a mut L> {}

    impl<'a, N: NetlistBase> NetlistBaseDecorator for DummyDecorator<&'a mut N> {}

    impl<'a, N: NetlistEdit> NetlistEditDecorator for DummyDecorator<&'a mut N> {}

    impl<'a, LN: L2NBase> L2NBaseDecorator for DummyDecorator<&'a mut LN> {}

    impl<'a, LN: L2NEdit> L2NEditDecorator for DummyDecorator<&'a mut LN> {}

    // Do some some operations with the decorated struct.
    let mut decorated_chip = DummyDecorator(&mut chip);
    let layer = decorated_chip.create_layer(0, 0);
    let cell = decorated_chip.create_cell("CELL".to_string().into());
    let geometry = Rect::new((0, 0), (10, 10));
    let shape = decorated_chip.insert_shape(&cell, &layer, geometry.into());
    let net = decorated_chip.get_net_of_shape(&shape);
    assert!(net.is_none());
}