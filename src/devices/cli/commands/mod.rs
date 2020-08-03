use crate::{
    devices::{
        bootloader::Bootloader,
        cli::{ArgumentIterator, Cli, Error, Name, RetrieveArgument},
    },
    hal::{flash, serial},
};
use ufmt::uwriteln;

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
                uwriteln!(cli.serial, "Starting Test...").ok().unwrap();
                bootloader.test_mcu_flash()?;
                bootloader.test_external_flash()?;
                uwriteln!(cli.serial, "Both Flash tests successful").ok().unwrap();
            }
            (true, false) => {
                uwriteln!(cli.serial, "Starting Test...").ok().unwrap();
                bootloader.test_mcu_flash()?;
                uwriteln!(cli.serial, "MCU flash test successful").ok().unwrap();
            }
            (false, true) => {
                uwriteln!(cli.serial, "Starting Test...").ok().unwrap();
                bootloader.test_external_flash()?;
                uwriteln!(cli.serial, "External flash test successful").ok().unwrap();
            }
            (false, false) => {
                return Err(Error::MissingArgument);
            }
        }
    },

    flash ["Stores a FW image in an external Bank."] (
        size: usize ["[0-2000]"],
        bank: u8 ["External Bank Index"],
        )
    {
        if size > 2000 {
            return Err(Error::ArgumentOutOfRange);
        }
        let exists = bootloader.external_banks().any(|b| b.index == bank);
        if exists {
            uwriteln!(cli.serial, "Starting raw read mode! [size] bytes will be read directly from now on.").ok().unwrap();
            bootloader.store_image(cli.serial.bytes().take(size as usize), size, bank)?;
            uwriteln!(cli.serial, "Image transfer complete!").ok().unwrap();
        } else {
            uwriteln!(cli.serial, "Index supplied does not correspond to an external bank.").ok().unwrap();
        }

    },

    format ["Erases a flash chip and initializes all headers to default values."] (
        mcu: bool ["Set to format MCU flash"],
        external: bool ["Set to format external flash"],
    ){
        match (mcu, external) {
            (true, true) => {
                uwriteln!(cli.serial, "Formatting...").ok().unwrap();
                bootloader.format_mcu_flash()?;
                uwriteln!(cli.serial, "MCU Flash formatted successfully.").ok().unwrap();
                bootloader.format_external_flash()?;
                uwriteln!(cli.serial, "Both Flash chips formatted successfully.").ok().unwrap();
            }
            (true, false) => {
                uwriteln!(cli.serial, "Formatting...").ok().unwrap();
                bootloader.format_mcu_flash()?;
                uwriteln!(cli.serial, "MCU Flash formatted successfully.").ok().unwrap();
            }
            (false, true) => {
                uwriteln!(cli.serial, "Formatting...").ok().unwrap();
                bootloader.format_external_flash()?;
                uwriteln!(cli.serial, "External Flash formatted successfully.").ok().unwrap();
            }
            (false, false) => {
                return Err(Error::MissingArgument);
            }
        }
    },

    banks ["Retrieves information from FW image banks."] (){
        uwriteln!(cli.serial, "MCU Banks:").ok().unwrap();
        for bank in bootloader.mcu_banks() {
            uwriteln!(cli.serial, "   - [{}] {} - Size: {}b",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size).ok().unwrap();
            if let Some(image) = bootloader.image_at_bank(bank.index) {
                uwriteln!(cli.serial, "        - [IMAGE] {} - Size: {}b - CRC: {} ",
                    if let Some(_) = image.name { "Placeholder Name" } else { "Anonymous" },
                    image.size,
                    image.crc).ok().unwrap();
            }

        }
        uwriteln!(cli.serial, "External Banks:").ok().unwrap();
        for bank in bootloader.external_banks() {
            uwriteln!(cli.serial, "   - [{}] {} - Size: {}b",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size).ok().unwrap();
            if let Some(image) = bootloader.image_at_bank(bank.index) {
                uwriteln!(cli.serial, "        - [IMAGE] {} - Size: {}b - CRC: {} ",
                    if let Some(_) = image.name { "Placeholder Name" } else { "Anonymous" },
                    image.size,
                    image.crc).ok().unwrap();
            }
        }
    },

    copy ["Copy an image from an external bank to an MCU bank."] (
           input: u8 ["External Bank index to copy from."],
           output: u8 ["MCU Bank index to copy to."],
        )
    {
        bootloader.copy_image(input, output)?;
        uwriteln!(cli.serial, "Copy success!").ok().unwrap();
    },

    boot ["Boot from a bootable MCU bank."] (
           bank: u8 ["Bootable MCU bank index."],
        )
    {
        uwriteln!(cli.serial, "Attempting to boot from bank {}", bank).ok().unwrap();
        bootloader.boot(bank)?;
    },

]);
