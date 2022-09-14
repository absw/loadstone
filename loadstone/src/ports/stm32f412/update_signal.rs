use crate::devices::update_signal::{self, UpdatePlan};
use blue_hal::stm32pac::RTC;

pub struct UpdatePlanner {
    rtc: RTC,
}

impl UpdatePlanner {
    pub fn new(rtc: RTC) -> Self {
        Self { rtc }
    }
}

impl update_signal::ReadUpdateSignal for UpdatePlanner {
    fn read_update_plan(&self) -> UpdatePlan {
        match self.rtc.bkpr[0].read().bits() {
            0x00000000 => UpdatePlan::None,
            0xFFFFFFFF => UpdatePlan::Any,
            0xFFFFFF00 => UpdatePlan::Serial,
            x => UpdatePlan::Index(x as u8),
        }
    }
}

impl update_signal::WriteUpdateSignal for UpdatePlanner {
    fn write_update_plan(&mut self, plan: UpdatePlan) {
        let bits = match plan {
            UpdatePlan::None => 0x00000000,
            UpdatePlan::Any => 0xFFFFFFFF,
            UpdatePlan::Serial => 0xFFFFFF00,
            UpdatePlan::Index(x) => x as u32,
        };
        self.rtc.bkpr[0].write(|w| unsafe { w.bits(bits) });
    }
}

/// Initializes the backup domain registers of the realtime clock, required for the update signal
/// to function.
pub fn initialize_rtc_backup_domain(rcc: &mut blue_hal::stm32pac::RCC, pwr: &mut blue_hal::stm32pac::PWR) {
    rcc.apb1enr.modify(|_, w| { w.pwren().set_bit() });
    pwr.csr.modify(|_, w| { w.bre().set_bit() });
    pwr.cr.modify(|_, w| { w.dbp().set_bit() });
    rcc.bdcr.modify(|_, w| {
        w.rtcen().set_bit()
        .rtcsel().bits(0b10)
    });
}
