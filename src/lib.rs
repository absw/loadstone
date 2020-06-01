#![feature(never_type)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
extern crate panic_semihosting; // logs messages to the host stderr

#[macro_use]
pub mod drivers;
#[macro_use]
pub mod hal;
pub mod pin_configuration;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn foo() {
        println!("tests work!");
        assert!(3 == 3);
    }
}
