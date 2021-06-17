use crate::devices::update_signal::{self, UpdatePlan};
use blue_hal::stm32pac::RTC;

pub struct UpdateSignal {
    rtc: RTC,
}

impl UpdateSignal {
    pub fn new(rtc: RTC) -> Self {
        Self { rtc }
    }
}

impl update_signal::UpdateSignal for UpdateSignal {
    fn update_plan(&self) -> UpdatePlan {
        match self.rtc.bkpr[0].read().bits() {
            0x00000000 => UpdatePlan::None,
            0xFFFFFFFF => UpdatePlan::Any,
            x => UpdatePlan::Index(x),
        }
    }
}
