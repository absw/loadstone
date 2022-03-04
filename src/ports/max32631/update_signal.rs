use crate::devices::update_signal::{ReadUpdateSignal, UpdatePlan, WriteUpdateSignal};

#[derive(Default)]
pub struct NullUpdatePlanner;

impl ReadUpdateSignal for NullUpdatePlanner {
    fn read_update_plan(&self) -> UpdatePlan { UpdatePlan::Any }
}

impl WriteUpdateSignal for NullUpdatePlanner {
    fn write_update_plan(&mut self, _plan: UpdatePlan) {}
}
