#![cfg_attr(test, allow(unused_attributes))]
#![cfg_attr(all(not(test), target_arch = "arm"), no_std)]
#![cfg_attr(target_arch = "arm", no_main)]

#[allow(unused_imports)]
use cortex_m_rt::{entry, exception};
pub const HEAP_SIZE_BYTES: usize = 8192;

pub const GREETING: &str =
    "--=Loadstone demo app CLI + Boot Manager=--\ntype `help` for a list of commands.";

#[cfg(all(target_arch = "arm", feature = "stm32f412_discovery"))]
#[entry]
fn main() -> ! {
    let heap_start = cortex_m_rt::heap_start() as usize;
    unsafe { loadstone_lib::ALLOCATOR.init(heap_start, HEAP_SIZE_BYTES) }

    use loadstone_lib::devices::boot_manager;
    let app = boot_manager::BootManager::new();
    app.run(GREETING);
}

#[cfg(all(target_arch = "arm", feature = "wgm160p"))]
#[entry]
fn main() -> ! {
    use loadstone_lib as _;
    loop {}
}

#[cfg(not(target_arch = "arm"))]
fn main() {}
