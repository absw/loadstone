//! GPIO configuration and alternate functions for the [stm32f412 discovery](../../../../../../../../documentation/hardware/discovery.pdf).
use crate::ports::pin_configuration::*;
use crate::hal::{serial::Write, time};
use crate::drivers::{
    stm32f4::gpio::{GpioExt, *},
    stm32f4::qspi::{self, mode, QuadSpi},
    stm32f4::rcc::Clocks,
    led::{MonochromeLed, LogicLevel},
    stm32f4::serial::{self, UsartExt},
    stm32f4::systick::{Tick, SysTick},
    stm32f4::flash,
    micron::n25q128a_flash::{self, MicronN25q128a},
};
use crate::stm32pac::{self, USART6};
use crate::devices::bootloader::Bootloader;
use core::marker::PhantomData;

// Flash pins and typedefs
type QspiPins = (Pb2<AF9>, Pg6<AF10>, Pf8<AF10>, Pf9<AF10>, Pf7<AF9>, Pf6<AF9>);
type Qspi = QuadSpi<QspiPins, mode::Single>;
type ExternalFlash = MicronN25q128a<Qspi, SysTick>;
type ExternalAddress = n25q128a_flash::Address;

// Serial pins and typedefs
type UsartPins = (Pg14<AF8>, Pg9<AF8>);
type Serial = serial::Serial<USART6, UsartPins>;
type PostLed = MonochromeLed<Pe1<Output<PushPull>>>;

impl Bootloader<ExternalFlash, flash::McuFlash, Serial, PostLed> {
    pub fn new() -> Self {
        let mut peripherals = stm32pac::Peripherals::take().unwrap();
        let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
        let gpiob = peripherals.GPIOB.split(&mut peripherals.RCC);
        let gpiog = peripherals.GPIOG.split(&mut peripherals.RCC);
        let gpiof = peripherals.GPIOF.split(&mut peripherals.RCC);
        let gpioe = peripherals.GPIOE.split(&mut peripherals.RCC);
        let post_led = MonochromeLed::new(gpioe.pe1, LogicLevel::Inverted);
        let clocks = Clocks::hardcoded(&peripherals.FLASH, peripherals.RCC);

        let systick = SysTick::new(cortex_peripherals.SYST, clocks);
        systick.wait(time::Seconds(1)); // Gives time for the flash chip to stabilize after powerup

        let serial_config = serial::config::Config::default().baudrate(time::Bps(115_200));
        let serial_pins = (gpiog.pg14, gpiog.pg9);
        let mut serial = peripherals.USART6.constrain(serial_pins, serial_config, clocks).unwrap();
        uprintln!(serial, "Initialising Secure Bootloader");

        let qspi_pins = (gpiob.pb2, gpiog.pg6, gpiof.pf8, gpiof.pf9, gpiof.pf7, gpiof.pf6);
        let qspi_config = qspi::Config::<mode::Single>::default().with_flash_size(24).unwrap();
        let qspi = Qspi::from_config(peripherals.QUADSPI, qspi_pins, qspi_config).unwrap();
        let external_flash = ExternalFlash::with_timeout(qspi, time::Milliseconds(500), systick).unwrap();
        let mcu_flash = flash::McuFlash::new(peripherals.FLASH).unwrap();

        Bootloader { external_flash, mcu_flash, post_led, serial }
    }
}
