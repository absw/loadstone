//! Convenience macros for the Bootloader project
#![macro_use]

/// Define and export a specific port module (transparently pulls
/// its namespace to the current one).
///
/// Used mostly to conveniently fit the module declaration and reexport
/// under a single configuration flag.
///
/// # Example
/// ```ignore
/// #[cfg(feature = "stm32_any")]
/// port!(stm32);
/// // Expands into:
/// pub mod stm32;
/// pub use self::stm32::*;
///
/// #[cfg(feature = "stm32_any")]
/// port!(stm32::flash as mcu_flash);
/// // Expands into:
/// pub mod stm32 { pub mod flash };
/// pub use self::stm32::flash as mcu_flash;
/// ```
#[macro_export]
macro_rules! port {
    ($mod:ident) => {
        pub mod $mod;
        pub use self::$mod::*;
    };
    ($mod:ident as $name:ident) => {
        pub mod $mod;
        pub use self::$mod as $name;
    };
    ($outer:ident::$inner:ident) => {
        pub mod $outer { pub mod $inner; }
        pub use self::$outer::$inner::*;
    };
    ($outer:ident::$inner:ident as $name:ident) => {
        pub mod $outer { pub mod $inner; }
        pub use self::$outer::$inner as $name;
    };
    ($outer:ident: [$($inner:ident,)+]) => {
        pub mod $outer {
        $(
            pub mod $inner;
        )+
        }
        $(
            pub use self::$outer::$inner;
        )+
    };
    ($outer:ident: [$($inner:ident as $name:ident)+,]) => {
        pub mod $outer {
        $(
            pub mod $inner;
        )+
        }
        $(
            pub use self::$outer::$inner as $name;
        )+
    };
}
