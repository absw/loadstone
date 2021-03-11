use crate::{
    devices::{
        boot_manager::BootManager,
        boot_metrics::BootPath,
        cli::{file_transfer::FileTransfer, ArgumentIterator, Cli, Error, Name, RetrieveArgument},
        image::{self, MAGIC_STRING},
        traits::{Flash, Serial},
    },
    error::Error as ApplicationError,
};
use blue_hal::uprintln;
use ufmt::uwriteln;

commands!( cli, boot_manager, names, helpstrings [

    help ["Displays a list of commands."] (command: Option<&str> ["Optional command to inspect."],) {
        cli.print_help(names, helpstrings, command)
    },

    banks ["Displays bank information"] (){
        uprintln!(cli.serial, "[{}] Banks:", MCUF::label());
        for bank in boot_manager.mcu_banks() {
            uwriteln!(cli.serial, "   - [{}] {} - Size: {}b{}",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size,
                if bank.is_golden { " - GOLDEN" } else { "" }).ok().unwrap();
        }

        if boot_manager.external_banks().count() > 0 {
            uprintln!(cli.serial, "[{}] Banks:", EXTF::label());
        }
        for bank in boot_manager.external_banks() {
            uwriteln!(cli.serial, "   - [{}] {} - Size: {}b{}",
                bank.index,
                if bank.bootable { "Bootable" } else { "Non-Bootable" },
                bank.size,
                if bank.is_golden { " - GOLDEN" } else { "" }).ok().unwrap();
        }
    },

    images ["Displays image information (WARNING: Slow)"] (){
        uprintln!(cli.serial, "[{}] Images:", MCUF::label());
        for bank in boot_manager.mcu_banks() {
            if let Ok(image) = image::image_at(&mut boot_manager.mcu_flash, bank) {
                uwriteln!(cli.serial, "Bank {} - [IMAGE] - Size: {}b - {}",
                    bank.index,
                    image.size(),
                    if image.is_golden() { " - GOLDEN" } else { "" }).ok().unwrap();
            }
        }
        if let Some(ref mut external_flash) = boot_manager.external_flash {
            uprintln!(cli.serial, "[{}] Images:", EXTF::label());
            for bank in boot_manager.external_banks.iter().cloned() {
                if let Ok(image) = image::image_at(external_flash, bank) {
                    uwriteln!(cli.serial, "Bank {} - [IMAGE] - Size: {}b - {}",
                        bank.index,
                        image.size(),
                        if image.is_golden() { " - GOLDEN" } else { "" }).ok().unwrap();
                }
            }
        }
    },

    flash ["Stores a FW image in a non-bootable bank."] (
        bank: u8 ["Bank index."],
        )
    {
        if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
            uprintln!(cli.serial, "Starting XMODEM mode! Send file with your XMODEM client.");
            boot_manager.store_image_external(cli.serial.blocks(None), bank)?;
            uprintln!(cli.serial, "Image transfer complete!");
        } else if let Some(bank) = boot_manager.mcu_banks().find(|b| b.index == bank) {
            if bank.bootable {
                uprintln!(cli.serial, "You can't erase the bootable image, it's what you are");
                uprintln!(cli.serial, "currently running! You can still corrupt its signature");
                uprintln!(cli.serial, "to force it to be invalid.");
                return Err(Error::ApplicationError(ApplicationError::BankInvalid));
            }
            uprintln!(cli.serial, "Starting XMODEM mode! Send file with your XMODEM client.");
            boot_manager.store_image_mcu(cli.serial.blocks(None), bank)?;
            uprintln!(cli.serial, "Image transfer complete!");
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to any bank.");
        }

    },

    corrupt_signature ["Corrupts the ECDSA signature of a specified image."] (
        bank: u8 ["Bank index."],
        )
    {


        if let Some(ref mut external_flash) = boot_manager.external_flash {
            if let Some(bank) = boot_manager.external_banks.iter().cloned().find(|b| b.index == bank) {
                let image = image::image_at(external_flash, bank)
                    .map_err(|_| Error::ApplicationError(ApplicationError::BankEmpty))?;
                let signature_location = image.location() + image.size() + MAGIC_STRING.len();
                let mut signature_bytes = [0u8; 64usize];
                nb::block!(external_flash.read(signature_location, &mut signature_bytes))
                    .map_err(|e| Error::ApplicationError(e.into()))?;
                signature_bytes[0] = !signature_bytes[0];
                nb::block!(external_flash.write(signature_location, &mut signature_bytes))
                    .map_err(|e| Error::ApplicationError(e.into()))?;
                uprintln!(cli.serial, "Flipped the first signature byte from {} to {}.", !signature_bytes[0], signature_bytes[0]);
            }
        } else if let Some(bank) = boot_manager.mcu_banks().find(|b| b.index == bank) {
            uprintln!(cli.serial, "Warning: Corrupting a signature in the MCU flash should work, but it might cause");
            uprintln!(cli.serial, "the application to crash.");
            let image = image::image_at(&mut boot_manager.mcu_flash, bank)
                .map_err(|_| Error::ApplicationError(ApplicationError::BankEmpty))?;
            let signature_location = image.location() + image.size() + MAGIC_STRING.len();
            let mut signature_bytes = [0u8; 64usize];
            nb::block!(boot_manager.mcu_flash.read(signature_location, &mut signature_bytes))
                .map_err(|e| Error::ApplicationError(e.into()))?;
            signature_bytes[0] = !signature_bytes[0];
            nb::block!(boot_manager.mcu_flash.write(signature_location, &mut signature_bytes))
                .map_err(|e| Error::ApplicationError(e.into()))?;
            uprintln!(cli.serial, "Flipped the first signature byte from {} to {}.", !signature_bytes[0], signature_bytes[0]);
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to any bank.");
            return Ok(());
        };
    },

    corrupt_body ["Corrupts a byte inside a specified external image."] (
        bank: u8 ["External bank index."],
        )
    {
        let external_flash = boot_manager.external_flash.as_mut()
            .ok_or(Error::ApplicationError(ApplicationError::NoExternalFlash))?;

        let bank = if let Some(bank) = boot_manager.external_banks.iter().cloned().find(|b| b.index == bank) {
            bank
        } else {
            uprintln!(cli.serial, "Index supplied does not correspond to an external bank.");
            return Ok(());
        };

        let image = image::image_at(external_flash, bank)
            .map_err(|_| Error::ApplicationError(ApplicationError::BankEmpty))?;

        let byte_location = image.location() + 1;
        let mut byte_buffer = [0u8];
        nb::block!(external_flash.read(byte_location, &mut byte_buffer)).map_err(|e| Error::ApplicationError(e.into()))?;
        byte_buffer[0] = !byte_buffer[0];
        nb::block!(external_flash.write(byte_location, &mut byte_buffer)).map_err(|e| Error::ApplicationError(e.into()))?;
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
            uprintln!(cli.serial, "[Boot Metrics]");
            match metrics.boot_path {
                BootPath::Direct => {
                    uprintln!(cli.serial, "* Application was booted directly from the MCU bank.");
                },
                BootPath::Restored { bank } => {
                    let bank_index = bank;
                    if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
                        uprintln!(cli.serial,
                            "* Application was first restored from bank {}{}, ([{}]) then booted.",
                            bank_index,
                            if bank.is_golden { " (GOLDEN)" } else {""},
                            EXTF::label(),
                        );
                    } else if let Some(bank) = boot_manager.mcu_banks().find(|b| b.index == bank) {
                        uprintln!(cli.serial,
                            "* Application was first restored from bank {}{}, ([{}]) then booted.",
                            bank_index,
                            if bank.is_golden { " (GOLDEN)" } else {""},
                            MCUF::label(),
                        );
                    }
                },
                BootPath::Updated { bank } => {
                    let bank_index = bank;
                    if let Some(bank) = boot_manager.external_banks().find(|b| b.index == bank) {
                        uprintln!(cli.serial,
                            "* Application was first updated from bank {}{}, ([{}]), then booted.",
                            bank_index,
                            if bank.is_golden { " (GOLDEN)" } else {""},
                            EXTF::label()
                        );
                    } else if let Some(bank) = boot_manager.mcu_banks().find(|b| b.index == bank) {
                        uprintln!(cli.serial,
                            "* Application was first updated from bank {}{}, ([{}]), then booted.",
                            bank_index,
                            if bank.is_golden { " (GOLDEN)" } else {""},
                            MCUF::label()
                        );
                    }
                },
            }
            uprintln!(cli.serial, "* Boot process took {} milliseconds.", metrics.boot_time_ms);
        } else {
            uprintln!(cli.serial, "Loadstone did not relay any boot metrics, or the boot metrics were corrupted.");
        }
    },

]);
