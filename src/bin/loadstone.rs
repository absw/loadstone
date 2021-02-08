#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
pub const HEAP_SIZE_BYTES: usize = 8192;


#[cfg(target_arch = "arm")]
#[entry]
fn main() -> ! {
    let heap_start = cortex_m_rt::heap_start() as usize;
    unsafe { loadstone_lib::ALLOCATOR.init(heap_start, HEAP_SIZE_BYTES) }

    use loadstone_lib::devices::bootloader::Bootloader;
    let bootloader = Bootloader::new();
    bootloader.run();
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
