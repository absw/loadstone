//! Concrete boot manager construction and flash bank layout
//! for the [stm32f412 discovery](../../../../loadstone/hardware/discovery.pdf).
use crate::devices::{boot_manager::BootManager, cli::Cli};
use blue_hal::{drivers::{micron::n25q128a_flash::MicronN25q128a, stm32f4::{qspi::{self, QuadSpi, mode}, rcc::Clocks, serial::{self, UsartExt}, systick::SysTick}}, hal::time, stm32pac::{self, USART6}};
use super::{bootloader::EXTERNAL_BANKS, pin_configuration::*};

// Flash pins and typedefs
type QspiPins = (Pb2<AF9>, Pg6<AF10>, Pf8<AF10>, Pf9<AF10>, Pf7<AF9>, Pf6<AF9>);
type Qspi = QuadSpi<QspiPins, mode::Single>;
type ExternalFlash = MicronN25q128a<Qspi, SysTick>;
type UsartPins = (Pg14<AF8>, Pg9<AF8>);
type Serial = serial::Serial<USART6, UsartPins>;

impl Default for BootManager<ExternalFlash, Serial> {
    fn default() -> Self { Self::new() }
}

impl BootManager<ExternalFlash, Serial> {
    pub fn new() -> Self {
        let mut peripherals = stm32pac::Peripherals::take().unwrap();
        let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
        let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
        let gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let gpiof = peripherals.GPIOF.split(&mut peripherals.RCC);
        let clocks = Clocks::hardcoded(peripherals.RCC);

        SysTick::init(cortex_peripherals.SYST, clocks);
        SysTick::wait(time::Seconds(1)); // Gives time for the flash chip to stabilize after powerup

        let serial_config = serial::config::Config::default().baudrate(time::Bps(115200));
        let serial_pins = (gpiog.pg14, gpiog.pg9);
        let serial = peripherals.USART6.constrain(serial_pins, serial_config, clocks).unwrap();
        let cli = Cli::new(serial).unwrap();

        let qspi_pins = (gpiob.pb2, gpiog.pg6, gpiof.pf8, gpiof.pf9, gpiof.pf7, gpiof.pf6);
        let qspi_config = qspi::Config::<mode::Single>::default().with_flash_size(24).unwrap();
        let qspi = Qspi::from_config(peripherals.QUADSPI, qspi_pins, qspi_config).unwrap();
        let external_flash = ExternalFlash::with_timeout(qspi, time::Milliseconds(5000)).unwrap();

        BootManager { external_flash, external_banks: &EXTERNAL_BANKS, cli: Some(cli), boot_metrics: None }
    }
}
