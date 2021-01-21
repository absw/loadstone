use crate::{
    devices::{
        boot_manager::BootManager,
        cli::{file_transfer::FileTransfer, ArgumentIterator, Cli, Error, Name, RetrieveArgument},
    },
    error::Error as ApplicationError,
};
use blue_hal::{
    hal::{flash, serial},
    uprintln,
};
use ufmt::uwriteln;

commands!( cli, boot_manager, names, helpstrings [

    help ["Displays a list of commands."] (command: Option<&str> ["Optional command to inspect."],) {
        cli.print_help(names, helpstrings, command)
    },

    flash ["Stores a FW image in an external bank."] (
        size: usize ["Image size in bytes"],
        bank: u8 ["External Bank Index"],
        )
    {
        if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
            if size > bank.size {
                return Err(Error::ArgumentOutOfRange);
            }
            uprintln!(cli.serial, "Starting XModem mode! Send file with your XModem client.");
            boot_manager.store_image(cli.serial.blocks(Some(10)), size, bank)?;
            uprintln!(cli.serial, "Image transfer complete!");
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to an external bank.");
        }

    },

    boot ["Restart, attempting to boot into a valid image if available."] ( )
    {
        uprintln!(cli.serial, "Restarting...");
        boot_manager.reset();
    },

]);
