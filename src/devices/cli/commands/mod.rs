use crate::{
    devices::cli::{Arguments, Cli, Error, Name, RetrieveArgument},
    hal::serial,
};

commands!( cli, names, helpstrings [
    help(test: Option<u32>,) ["Displays a list of commands."] { cli.print_help(names, helpstrings) },
]);

//help(a: MyEnum, b: Option<i32>)
