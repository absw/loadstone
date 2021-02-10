#[repr(C)]
#[derive(Clone)]
pub struct BootMetrics {
    pub test: u32,
}

/// # Safety
///
/// Horrendously unsafe. Simply returns a block at end of RAM reinterpreted as an arbitrary struct.
/// Only useful right before bootstrapping the app to leave some metrics information for it to
/// consume.
///
/// This *will* clobber data so it must only be called immediately before jumping into the target
/// application.
pub unsafe fn boot_metrics_mut() -> &'static mut BootMetrics {
    let ram_end = 0x20010000;
    let boot_metrics_raw: *mut BootMetrics
        = core::mem::transmute::<usize, *mut BootMetrics>(ram_end - core::mem::size_of::<BootMetrics>());
    boot_metrics_raw.as_mut().unwrap()
}

/// # Safety
///
/// Horrendously unsafe. Simply returns a block at end of RAM reinterpreted as an arbitrary struct.
/// Only useful right after bootstrapping the app, to retrieve metrics information before having a
/// chance to clobber it.
pub unsafe fn boot_metrics() -> &'static BootMetrics {
    boot_metrics_mut()
}
