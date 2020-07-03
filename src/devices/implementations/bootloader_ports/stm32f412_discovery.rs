use crate::{
    devices::implementations::flash::micron_n25q128a::MicronN25q128a,
    drivers::{
        gpio::{GpioExt, *},
        qspi::{mode, QuadSpi, self},
        rcc::RccExt,
        serial::{self, UsartAf, UsartExt},
    },
    hal,
    pin_configuration::*,
    stm32pac::{Peripherals, USART6},
};

// Flash pins and typedefs
type QspiPins = (Pb2<AF9>, Pg6<AF10>, Pf8<AF10>, Pf9<AF10>);
type Qspi = QuadSpi<QspiPins, mode::Single>;
type Flash = MicronN25q128a<Qspi>;

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
        let gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let clocks = peripherals
            .RCC
            .constrain()
            .sysclk(hal::time::MegaHertz(180))
            .hclk(hal::time::MegaHertz(84))
            .pclk1(hal::time::MegaHertz(42))
            .pclk2(hal::time::MegaHertz(84))
            .require_pll48clk()
            .freeze();
        let serial_config = serial::config::Config::default().baudrate(hal::time::Bps(115_200));
        let serial_pins = (gpiog.pg14, gpiog.pg9);
        let serial = peripherals.USART6.constrain(serial_pins, serial_config, clocks);

        //let qspi_config = qspi::Config::default();
        //let flash = Flash::new(Qspi::from_config(qspi_config));

        unimplemented!();
    }
}
