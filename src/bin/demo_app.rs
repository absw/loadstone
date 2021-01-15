#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
use loadstone_lib::devices::boot_manager;

#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    let app = boot_manager::BootManager {};
    loop {}
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
