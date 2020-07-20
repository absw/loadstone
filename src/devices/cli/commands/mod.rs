use crate::devices::cli::{Arguments, Cli, Name};
use crate::hal::serial;

commands!( cli [
    help() { uprintln!(cli.serial, "Hello world!") },
]);
