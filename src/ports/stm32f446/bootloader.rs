use crate::devices::bootloader::Bootloader;
use blue_hal::{
    hal::null::{NullError, NullFlash, NullSerial, NullSystick},
    drivers::stm32f4::{
        flash::{self, McuFlash},
    },
};
use crate::devices::image::CrcImageReader as ImageReader;
use super::update_signal::NullUpdatePlanner;
use crate::error;

impl error::Convertible for NullError {
    fn into(self) -> error::Error { panic!() }
}

impl error::Convertible for flash::Error {
    fn into(self) -> error::Error {
        todo!()
    }
}

impl Bootloader<NullFlash, McuFlash, NullSerial, NullSystick, ImageReader, NullUpdatePlanner> {
    pub fn new() -> Self {
        todo!()
    }
}
