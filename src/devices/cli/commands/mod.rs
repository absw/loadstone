use crate::{
    devices::{
        bootloader::Bootloader,
        cli::{ArgumentIterator, Cli, Error, Name, RetrieveArgument},
    },
    hal::{flash, serial},
};

commands!( cli, bootloader, names, helpstrings [

    help ["Displays a list of commands."] (command: Option<&str> ["Optional command to inspect."],) {
        cli.print_help(names, helpstrings, command)
    },

    test ["Tests various elements of the bootloader."](
        mcu: bool ["Set to test MCU flash"],
        external: bool ["Set to test external flash"],
        complex: bool ["Set to perform complex tests"],
    ){
        match (mcu, external) {
            (true, true) => {
                uprintln!(cli.serial, if complex { "Starting Complex Test..." } else { "Starting Simple Test..." });
                bootloader.test_mcu_flash(complex)?;
                bootloader.test_external_flash(complex)?;
                uprintln!(cli.serial, "Both Flash tests successful");
            }
            (true, false) => {
                uprintln!(cli.serial, if complex { "Starting Complex Test..." } else { "Starting Simple Test..." });
                bootloader.test_mcu_flash(complex)?;
                uprintln!(cli.serial, "MCU flash test successful");
            }
            (false, true) => {
                uprintln!(cli.serial, if complex { "Starting Complex Test..." } else { "Starting Simple Test..." });
                bootloader.test_external_flash(complex)?;
                uprintln!(cli.serial, "External flash test successful");
            }
            (false, false) => {
                return Err(Error::MissingArgument);
            }
        }
    },

    flash ["Stores a FW image in external Flash."] (size: u32 ["[0-500]"],) {
        if size > 500 {
            return Err(Error::ArgumentOutOfRange);
        }
        uprintln!(cli.serial, "Starting raw read mode! [size] bytes will be read directly from now on.");
        bootloader.store_image(cli.serial.bytes().take(size as usize))?;
        uprintln!(cli.serial, "Image transfer complete!");
    },
]);
