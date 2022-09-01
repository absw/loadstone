//! Metrics relayed to the application by Loadstone.
//!
//! Immediately preceding the jump to a target image, Loadstone stores
//! a collection of metrics in a designated section of RAM. The application
//! is free to ignore these or collect them for display, reflection on the
//! boot process, or logging. It's important for the application to collect
//! these metrics immediately, as they exist in an untracked section of
//! memory where they can be quickly clobbered by stack variables.

/// Collection of boot metrics relayed by Loadstone to the booted application.
#[repr(C)]
#[derive(Clone)]
pub struct BootMetrics {
    /// Magic string to ensure the boot metrics' integrity when read. Must
    /// be equal to [`BOOT_MAGIC_START`] when read to guarantee validity.
    pub boot_magic_start: u32,
    /// The actions taken by Loadstone that ultimately led to an image being
    /// booted.
    pub boot_path: BootPath,
    /// Time from construction of Loadstone's driver suite to the target image
    /// being booted.
    pub boot_time_ms: Option<u32>,
    /// Magic string to ensure the boot metrics' integrity when read. Must
    /// be equal to [`BOOT_MAGIC_END`] when read to guarantee validity.
    pub boot_magic_end: u32,
}

/// Bit pattern that should mark the start of a valid boot metrics struct.
pub const BOOT_MAGIC_START: u32 = 0xDEADBEEF;
/// Bit pattern that should mark the end of a valid boot metrics struct.
pub const BOOT_MAGIC_END: u32 = 0xCAFEBABE;

/// Actions taken by Loadstone that ultimately led to an image being booted.
#[repr(C)]
#[derive(Clone)]
pub enum BootPath {
    /// The image was booted directly from the main MCU flash bank, as there
    /// was no newer image to supersede it.
    Direct,
    /// The image was initially restored from an external bank, then booted.
    Restored { bank: u8 },
    /// The image was initially updated from an external bank, then booted.
    Updated { bank: u8 },
}

impl Default for BootMetrics {
    fn default() -> Self {
        Self {
            boot_magic_start: BOOT_MAGIC_START,
            boot_path: BootPath::Direct,
            boot_time_ms: None,
            boot_magic_end: BOOT_MAGIC_END,
        }
    }
}

impl BootMetrics {
    /// The boot metrics struct is valid. This allows the application to verify that the metrics
    /// read directly from unstructed RAM has not been clobbered.
    pub fn is_valid(&self) -> bool {
        self.boot_magic_start == BOOT_MAGIC_START && self.boot_magic_end == BOOT_MAGIC_END
    }
}

/// Reinterprets an arbitrary memory range as a mutable boot metrics struct.
///
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
    let boot_metrics_raw: *mut BootMetrics = core::mem::transmute::<usize, *mut BootMetrics>(
        ram_end - core::mem::size_of::<BootMetrics>(),
    );
    boot_metrics_raw.as_mut().unwrap()
}

/// Reinterprets an arbitrary memory range as an immmutable boot metrics struct.
///
/// # Safety
///
/// Horrendously unsafe. Simply returns a block at end of RAM reinterpreted as an arbitrary struct.
/// Only useful right after bootstrapping the app, to retrieve metrics information before having a
/// chance to clobber it.
pub unsafe fn boot_metrics() -> &'static BootMetrics { boot_metrics_mut() }
