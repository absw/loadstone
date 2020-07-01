use crate::devices::implementations::flash::micron_n25q128a::MicronN25q128a;
use crate::{drivers::spi::{self, SpiAf}, pin_configuration::*};
use crate::{drivers::{serial::{UsartAf, Serial}, gpio::{PushPull, Output}}, stm32pac::SPI1};
use crate::stm32pac::USART6;

type SpiPins = (Pa6<SpiAf>, Pa7<SpiAf>, Pa5<SpiAf>);
type Spi = spi::Spi<SPI1, SpiPins, u8>;
type FlashChipSelect = Pg6<Output<PushPull>>;

type UsartPins = (Pg14<UsartAf>, Pg9<UsartAf>);

pub struct Bootloader {
    flash: MicronN25q128a<Spi, FlashChipSelect>,
    serial: Serial<USART6, UsartPins>,
}
