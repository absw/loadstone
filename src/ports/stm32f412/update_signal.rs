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

impl update_signal::ReadUpdateSignal for UpdateSignal {
    fn read_update_plan(&self) -> UpdatePlan {
        match self.rtc.bkpr[0].read().bits() {
            0x00000000 => UpdatePlan::None,
            0xFFFFFFFF => UpdatePlan::Any,
            x => UpdatePlan::Index(x as u8),
        }
    }
}

pub struct UpdateSignalWriter {
    rtc: RTC,
}

impl UpdateSignalWriter {
    pub fn new(rtc: RTC) -> Self {
        Self { rtc }
    }
}

impl update_signal::WriteUpdateSignal for UpdateSignalWriter {
    fn write_update_plan(&mut self, plan: UpdatePlan) {
        let bits = match plan {
            UpdatePlan::None => 0x00000000,
            UpdatePlan::Any => 0xFFFFFFFF,
            UpdatePlan::Index(x) => x as u32,
        };
        self.rtc.bkpr[0].write(|w| unsafe { w.bits(bits) });
    }
}
