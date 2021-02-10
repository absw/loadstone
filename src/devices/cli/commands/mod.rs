use crate::{
    devices::{
        boot_manager::BootManager,
        boot_metrics::BootPath,
        cli::{file_transfer::FileTransfer, ArgumentIterator, Cli, Error, Name, RetrieveArgument},
        image::{self, MAGIC_STRING},
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

    banks ["Displays external bank information"] (){
        uprintln!(cli.serial, "External Banks:");
        for bank in boot_manager.external_banks() {
            uwriteln!(cli.serial, "   - [{}] {} - Size: {}b{}",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size,
                if bank.is_golden { " - GOLDEN" } else { "" }).ok().unwrap();
        }
    },

    images ["Displays external image information (WARNING: Slow)"] (){
        uprintln!(cli.serial, "External images:");
        for bank in boot_manager.external_banks() {
            if let Ok(image) = image::image_at(&mut boot_manager.external_flash, bank) {
                uwriteln!(cli.serial, "Bank {} - [IMAGE] - Size: {}b - {}",
                    bank.index,
                    image.size(),
                    if image.is_golden() { " - GOLDEN" } else { "" }).ok().unwrap();
            }

        }
    },

    flash ["Stores a FW image in an external bank."] (
        bank: u8 ["External bank index."],
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

    corrupt_signature ["Corrupts the ECDSA signature of a specified external image."] (
        bank: u8 ["External bank index."],
        )
    {
        let bank = if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
            bank
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to an external bank.");
            return Ok(());
        };

        let image = image::image_at(&mut boot_manager.external_flash, bank)
            .map_err(|_| Error::ApplicationError(ApplicationError::BankEmpty))?;

        let signature_location = image.location() + image.size() + MAGIC_STRING.len();
        let mut signature_bytes = [0u8; 64usize];
        nb::block!(boot_manager.external_flash.read(signature_location, &mut signature_bytes))
            .map_err(|e| Error::ApplicationError(e.into()))?;
        signature_bytes[0] = !signature_bytes[0];
        nb::block!(boot_manager.external_flash.write(signature_location, &mut signature_bytes))
            .map_err(|e| Error::ApplicationError(e.into()))?;
        uprintln!(cli.serial, "Flipped the first signature byte from {} to {}.", !signature_bytes[0], signature_bytes[0]);
    },

    corrupt_body ["Corrupts a byte inside a specified external image."] (
        bank: u8 ["External bank index."],
        )
    {
        let bank = if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
            bank
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to an external bank.");
            return Ok(());
        };

        let image = image::image_at(&mut boot_manager.external_flash, bank)
            .map_err(|_| Error::ApplicationError(ApplicationError::BankEmpty))?;

        let byte_location = image.location() + 1;
        let mut byte_buffer = [0u8];
        nb::block!(boot_manager.external_flash.read(byte_location, &mut byte_buffer)).map_err(|e| Error::ApplicationError(e.into()))?;
        byte_buffer[0] = !byte_buffer[0];
        nb::block!(boot_manager.external_flash.write(byte_location, &mut byte_buffer)).map_err(|e| Error::ApplicationError(e.into()))?;
        uprintln!(cli.serial, "Flipped an application byte byte from {} to {}.", !byte_buffer[0], byte_buffer[0]);
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

    metrics ["Displays boot process metrics relayed by Loadstone."] ( )
    {
        if let Some(metrics) = &boot_manager.boot_metrics {
            match metrics.boot_path {
                uprintln!(cli.serial, "[Boot Metrics]");
                BootPath::Direct => {
                    uprintln!(cli.serial, "* Application was booted directly from the MCU bank.");
                },
                BootPath::Restored { bank } => {
                    uprintln!(cli.serial, "* Application was first restored from bank {}, then booted.", bank);
                },
            }
            uprintln!(cli.serial, "* Boot process took {} milliseconds.", metrics.boot_time_ms);
        } else {
            uprintln!(cli.serial, "Loadstone did not relay any boot metrics, or the boot metrics were corrupted.");
        }
    },

]);
