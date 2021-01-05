use crate::{
    devices::{
        bootloader::Bootloader,
        cli::{file_transfer::FileTransfer, ArgumentIterator, Cli, Error, Name, RetrieveArgument},
    },
    error::Error as BootloaderError,
};
use blue_hal::{hal::{flash, serial}, uprintln};
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

    flash ["Stores a FW image in an external Bank."] (
        size: usize ["Image size in bytes"],
        bank: u8 ["External Bank Index"],
        )
    {
        if let Some(bank) = bootloader.external_banks().find(|b| b.index == bank) {
            if size > bank.size {
                return Err(Error::ArgumentOutOfRange);
            }
            uprintln!(cli.serial, "Starting XModem mode! Send file with your XModem client.");
            bootloader.store_image(cli.serial.blocks(), size, bank)?;
            uprintln!(cli.serial, "Image transfer complete!");
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to an external bank.");
        }

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
        uprintln!(cli.serial, "External Banks:");
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
        uprintln!(cli.serial, "Copy success!");
    },

    boot ["Restart, attempting to boot into a valid image if available."] ( )
    {
        uprintln!(cli.serial, "Restarting...");
        bootloader.reset();
    },

]);
