use crate::{
    devices::cli::{Arguments, Cli, Error, Name, RetrieveArgument},
    hal::serial,
};

commands!( cli, names, helpstrings [
    help()
        ["Displays a list of commands."]
        { cli.print_help(names, helpstrings) },
    sample_command(_first: bool ["Optional Flag"], _second: u32 ["Numbers (0-100)"], _third: Option<u32> ["Optional thing (3-5)"],)
        ["A test command."]
        { uprintln!(cli.serial, "Hi!") },
]);

//help(a: MyEnum, b: Option<i32>)
