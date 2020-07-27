use crate::{
    devices::{
        bootloader::Bootloader,
        cli::{ArgumentIterator, Cli, Error, Name, RetrieveArgument},
    },
    hal::{flash, serial},
};

use arrayvec::ArrayString;
use core::fmt::Write;

commands!( cli, bootloader, names, helpstrings [

    help ["Displays a list of commands."] (command: Option<&str> ["Optional command to inspect."],) {
        cli.print_help(names, helpstrings, command)
    },

    test ["Tests various elements of the bootloader."] (
        mcu: bool ["Set to test MCU flash"],
        external: bool ["Set to test external flash"],
    ) {
        match (mcu, external) {
            (true, true) => {
                uprintln!(cli.serial, "Starting Test...");
                bootloader.test_mcu_flash()?;
                bootloader.test_external_flash()?;
                uprintln!(cli.serial, "Both Flash tests successful");
            }
            (true, false) => {
                uprintln!(cli.serial, "Starting Test...");
                bootloader.test_mcu_flash()?;
                uprintln!(cli.serial, "MCU flash test successful");
            }
            (false, true) => {
                uprintln!(cli.serial, "Starting Test...");
                bootloader.test_external_flash()?;
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

    format ["Erases a flash chip and initializes all headers to default values."] (
        mcu: bool ["Set to format MCU flash"],
        external: bool ["Set to format external flash"],
    ){
        match (mcu, external) {
            (true, true) => {
                uprintln!(cli.serial, "Formatting...");
                bootloader.format_mcu_flash()?;
                uprintln!(cli.serial, "MCU Flash formatted successfully.");
                bootloader.format_external_flash()?;
                uprintln!(cli.serial, "Both Flash chips formatted successfully.");
            }
            (true, false) => {
                uprintln!(cli.serial, "Formatting...");
                bootloader.format_mcu_flash()?;
                uprintln!(cli.serial, "MCU Flash formatted successfully.");
            }
            (false, true) => {
                uprintln!(cli.serial, "Formatting...");
                bootloader.format_external_flash()?;
                uprintln!(cli.serial, "External Flash formatted successfully.");
            }
            (false, false) => {
                return Err(Error::MissingArgument);
            }
        }
    },

    banks ["Retrieves information from FW image banks."] (){
        uprintln!(cli.serial, "MCU Banks:");
        let mut text = ArrayString::<[_; 64]>::new();
        for bank in bootloader.mcu_banks() {
            write!(text, "   - [{}] {} - Size: {}b",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size).expect("Not enough space to format bank string description.");
            uprintln!(cli.serial, text);
            text.clear();
            if let Some(image) = bootloader.image_at_bank(bank.index) {
                write!(text, "      * [IMAGE] {} - Size: {}b - CRC: {} ",
                    if let Some(_) = image.name { "Placeholder Name" } else { "Anonymous" },
                    image.size,
                    image.crc).expect("Not enough space to format image description");
                uprintln!(cli.serial, text);
                text.clear();
            }

        }
        uprintln!(cli.serial, "External Banks:");
        for bank in bootloader.external_banks() {
            write!(text, "   - [{}] {} - Size: {}b",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size).expect("Not enough space to format bank string description.");
            uprintln!(cli.serial, text);
            text.clear();
            if let Some(image) = bootloader.image_at_bank(bank.index) {
                write!(text, "      * [IMAGE] {} - Size: {}b - CRC: {} ",
                    if let Some(_) = image.name { "Placeholder Name" } else { "Anonymous" },
                    image.size,
                    image.crc).expect("Not enough space to format image description");
                uprintln!(cli.serial, text);
                text.clear();
            }
        }
    },
]);
