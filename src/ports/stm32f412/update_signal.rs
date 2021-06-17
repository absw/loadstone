use crate::devices::update_signal::*;

pub struct FixedUpdateSignal {
    plan: UpdatePlan,
}

impl FixedUpdateSignal {
    pub fn new(plan: UpdatePlan) -> Self {
        Self { plan }
    }
}

impl UpdateSignal for FixedUpdateSignal {
    fn update_plan(&self) -> UpdatePlan { self.plan }
}
