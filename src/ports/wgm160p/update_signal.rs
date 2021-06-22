use crate::devices::update_signal::{ReadUpdateSignal, UpdatePlan};

#[derive(Default)]
pub struct NullUpdateSignal;

impl ReadUpdateSignal for NullUpdateSignal {
    fn read_update_plan(&self) -> UpdatePlan { UpdatePlan::None }
}
