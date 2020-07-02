use crate::{
    devices::implementations::flash::micron_n25q128a::MicronN25q128a,
    drivers::{
        gpio::{GpioExt, Output, PushPull},
        rcc::RccExt,
        serial::{self, UsartAf},
        spi::{self, SpiAf},
    },
    hal,
    pin_configuration::*,
    stm32pac::{Peripherals, SPI1, USART6},
};

// Flash pins and typedefs
type SpiPins = (Pa6<SpiAf>, Pa7<SpiAf>, Pa5<SpiAf>);
type Spi = spi::Spi<SPI1, SpiPins, u8>; // TODO replace with QuadSPI
type FlashChipSelect = Pg6<Output<PushPull>>;
type Flash = MicronN25q128a<Spi, FlashChipSelect>;

// Serial pins and typedefs
type UsartPins = (Pg14<UsartAf>, Pg9<UsartAf>);
type Serial = serial::Serial<USART6, UsartPins>;

/// Top level Bootloader type for the stm32f412 Discovery board
pub struct Bootloader {
    _flash: Flash,
    _serial: Serial,
}

impl Bootloader {
    pub fn new(mut peripherals: Peripherals) -> Bootloader {
        let _gpioa = peripherals.GPIOA.split(&mut peripherals.RCC);
        let _gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let _clocks = peripherals
            .RCC
            .constrain()
            .sysclk(hal::time::MegaHertz(180))
            .hclk(hal::time::MegaHertz(84))
            .pclk1(hal::time::MegaHertz(42))
            .pclk2(hal::time::MegaHertz(84))
            .require_pll48clk()
            .freeze();

        let _serial_config = serial::config::Config::default().baudrate(hal::time::Bps(115_200));

        unimplemented!();
    }
}
