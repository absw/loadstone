use crate::devices::update_signal::{self, UpdatePlan};

pub struct UpdateSignal;

impl update_signal::UpdateSignal for UpdateSignal {
    fn update_plan(&self) -> UpdatePlan {
        todo!()
    }
}
