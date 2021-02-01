use crate::{
    devices::{
        boot_manager::BootManager,
        cli::{
            file_transfer::FileTransfer, ArgumentIterator, Cli, Error, Name, RetrieveArgument,
            GREETING,
        },
        image,
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
                uwriteln!(cli.serial, "Bank {} - [IMAGE] - Size: {}b - CRC: {}{}",
                    bank.index,
                    image.size(),
                    image.crc(),
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

    corrupt_crc ["Corrupts the CRC of a specified external image."] (
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

        let crc_location = image.location() + image.size();
        let mut crc_bytes = [0u8; 4usize];
        nb::block!(boot_manager.external_flash.read(crc_location, &mut crc_bytes)).map_err(|e| Error::ApplicationError(e.into()))?;
        crc_bytes[0] = !crc_bytes[0];
        nb::block!(boot_manager.external_flash.write(crc_location, &mut crc_bytes)).map_err(|e| Error::ApplicationError(e.into()))?;
        uprintln!(cli.serial, "Flipped the first CRC byte from {} to {}.", !crc_bytes[0], crc_bytes[0]);
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

    corrupt_mcu_image ["Corrupts greeting text in the current demo image."] ()
    {
        let greeting_address_raw = GREETING.as_bytes()[0] as *const u8 as usize;
        let aligned_greeting: MCUF::Address = (greeting_address_raw + (4 - greeting_address_raw % 4)).into();
        // Safety: Thoroughly unsafe as it modifies the current binary image live. Naturally, it is going
        // to make a significant, unpredictable change to the running application. Nothing should be
        // expected to work after this, hence why it recommends following with a restart.
        unsafe {
            nb::block!(boot_manager.mcu_flash.unlimited_write(
                    aligned_greeting,
                    "CORRUPTED: YOU SHOULD NOT BE READING THIS\r\n".as_bytes()))
                .map_err(|e| Error::ApplicationError(e.into()))?;
        }
        uprintln!(cli.serial, "Ỹ̷̙̜̼̮̬̱̪͙̈́͑͆̂̑̕o̷̭̠̪͉̞͔͛̓̍̔ŭ̷̻͓̳̳̪͑͗̈͋͛ ̵͔͓̯͒̋̑́̏̇͒͘s̵̨̬̜̤̰͂͐̇̊h̴̨̲̪̟͉̗̣̝̔͆̋̃͑ő̵͕̠͒͝ư̶̰͙̼̪͔͋͆̂̃̈̓͝l̶̢̠̤͕̹̽̾̇͌d̴̨̗̺͔̀ ̷̡̡̰̮̪̽ͅņ̷̻͙̟͓̣̖̈̐̿͐̆͆ǫ̶̧̦͎̜͙̼̤̋̒t̶̝͉͔̘̤͖͛̀̀͆̃̈̊ ̵̭͓̓́͆h̵̯͈̭̳̑̍̇ā̸̦̘̝v̸̩̬̦̘͆̍͑ͅȩ̵̣͚̾ ̸̛͔̮̫͖̤͕̱̄̑̇͘d̷̡̼̘͚͈͂͜o̸̢̹̲̲̍͛͒̑̐̕͠͝n̶̢̠̞̞̰̞͓̓̓͂̒͝e̴͇͋̑͘͜ ̷̡̜͙̰̘͐̿̇̆̄̕ṱ̵̢̦̫̲͍̓̃̇h̵̡͚̟͕̊̅́̎͌͐̚ä̶̢̤͓̻̗̭̣̬́t̸̡̢͖̝̫̖͓̭̽͊͐͒̌̈́");
        uprintln!(cli.serial, "Image may now behave unpredictably. Please restart.");
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
