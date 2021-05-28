
#[macro_use]
use crate::flyweight as fw;
use crate::flyweight::*;
use crate::{impl_flyweight_container, impl_template};

#[derive(Default)]
pub struct NetlistLayout {
    container: fw::FlyWeightContainer<Cell, CellInstance>
}

impl NetlistLayout {
    impl_flyweight_container! {Cell, CellInstance}
}

impl fw::FlyWeightContainerTrait<Cell, CellInstance> for NetlistLayout {
    fn fwc(&self) -> &fw::FlyWeightContainer<Cell, CellInstance> {
        &self.container
    }

    fn fwc_mut(&mut self) -> &mut fw::FlyWeightContainer<Cell, CellInstance> {
        &mut self.container
    }
}

pub struct Cell {
    tpl: fw::Template<Cell, CellInstance>
}

impl Cell {
    impl_template! {Cell, CellInstance}
}

impl TemplateTrait<Cell, CellInstance> for Cell {
    fn tpl(&self) -> &fw::Template<Cell, CellInstance> {
        &self.tpl
    }

    fn tpl_mut(&mut self) -> &mut fw::Template<Cell, CellInstance> {
        &mut self.tpl
    }

    fn new(t: fw::Template<Cell, CellInstance>) -> Self {
        Cell { tpl: t }
    }
}

pub struct CellInstance {
    inst: fw::Instance<Cell, CellInstance>
}

impl CellInstance {
    // impl_instance! {Circuit, CircuitInstance}
}

impl InstanceTrait<Cell, CellInstance> for CellInstance {
    fn inst(&self) -> &fw::Instance<Cell, CellInstance> {
        &self.inst
    }

    fn inst_mut(&mut self) -> &mut fw::Instance<Cell, CellInstance> {
        &mut self.inst
    }

    fn new(i: fw::Instance<Cell, CellInstance>) -> Self {
        CellInstance { inst: i }
    }
}