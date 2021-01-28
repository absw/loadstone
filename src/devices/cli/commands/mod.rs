use crate::{devices::{boot_manager::BootManager, cli::{file_transfer::FileTransfer, ArgumentIterator, Cli, Error, Name, RetrieveArgument}, image}, error::Error as ApplicationError};
use blue_hal::{
    hal::{flash, serial},
    uprintln,
};
use ufmt::uwriteln;

commands!( cli, boot_manager, names, helpstrings [

    help ["Displays a list of commands."] (command: Option<&str> ["Optional command to inspect."],) {
        cli.print_help(names, helpstrings, command)
    },

    banks ["Displays external bank information"] (){
        uprintln!(cli.serial, "External Banks:");
        for bank in boot_manager.external_banks() {
            uwriteln!(cli.serial, "   - [{}] {} - Size: {}b{}",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size,
                if bank.is_golden { "- GOLDEN" } else { "" }).ok().unwrap();
        }
    },

    images ["Displays external image information (WARNING: Slow)"] (){
        uprintln!(cli.serial, "External images:");
        for bank in boot_manager.external_banks() {
            if let Ok(image) = image::image_at(&mut boot_manager.external_flash, bank) {
                uwriteln!(cli.serial, "        - [IMAGE] - Size: {}b - CRC: {} ",
                    image.size(),
                    image.crc()).ok().unwrap();
            }

        }
    },

    flash ["Stores a FW image in an external bank."] (
        bank: u8 ["External Bank Index"],
        )
    {
        if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
            uprintln!(cli.serial, "Starting XModem mode! Send file with your XModem client.");
            boot_manager.store_image(cli.serial.blocks(Some(10)), bank)?;
            uprintln!(cli.serial, "Image transfer complete!");
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to an external bank.");
        }

    },

    format ["Formats external flash."] ()
    {
        uprintln!(cli.serial, "Formatting external flash...");
        boot_manager.format_external()?;
        uprintln!(cli.serial, "Done formatting!");
    },

    boot ["Restart, attempting to boot into a valid image if available."] ( )
    {
        uprintln!(cli.serial, "Restarting...");
        boot_manager.reset();
    },

]);
