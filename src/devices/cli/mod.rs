//! Generic command line interface.
//!
//! This module contains functionality for the CLI, except
//! for construction which is implementation-specific so is
//! handled in the `port` module.

#![macro_use]
use crate::error::Error as ApplicationError;
use blue_hal::{
    hal::serial::{self, Read},
    uprint, uprintln,
    utilities::{buffer::TryCollectSlice, iterator::Unique},
};
use core::str::{from_utf8, SplitWhitespace};
use nb::block;
use ufmt::{uwrite, uwriteln};

use super::{
    boot_manager::BootManager,
    traits::{Flash, Serial},
};

pub mod file_transfer;

const PROMPT: &str = "\n> ";
const BUFFER_SIZE: usize = 256;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    CommandEmpty,
    CommandUnknown,
    UnexpectedArguments,
    ArgumentOutOfRange,
    MalformedArguments,
    MissingArgument,
    CharactersNotAllowed,
    BadCommandEncoding,
    DuplicateArguments,
    SerialBufferOverflow,
    SerialReadError,
    ApplicationError(ApplicationError),
}

impl From<ApplicationError> for Error {
    fn from(e: ApplicationError) -> Self { Error::ApplicationError(e) }
}

pub const DEFAULT_GREETING: &str =
    "--=Loadstone demo app CLI + Boot Manager=--";

/// Command line interface struct, generic over a serial driver. Offers a collection of commands
/// to interact with the MCU and external flash chips and retrieve Loadstone boot metrics.
pub struct Cli<S: serial::ReadWrite> {
    serial: S,
    greeted: bool,
    needs_prompt: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Argument<'a> {
    Single(&'a str),
    Pair(&'a str, &'a str),
}

type Name<'a> = &'a str;

impl<'a> Argument<'a> {
    fn name(&self) -> Name {
        match self {
            Argument::Single(n) => n,
            Argument::Pair(n, _) => n,
        }
    }
}

#[derive(Clone)]
struct ArgumentIterator<'a> {
    tokens: SplitWhitespace<'a>,
}

impl<'a> Iterator for ArgumentIterator<'a> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(token) = self.tokens.next() {
            let mut split = token.split(ARGUMENT_SEPARATOR);
            match split.clone().count() {
                2 => return Some(Argument::Pair(split.next().unwrap(), split.next().unwrap())),
                1 => return Some(Argument::Single(split.next().unwrap())),
                _ => (),
            }
        }
        None
    }
}

trait Parsable<'a>: Sized {
    fn parse(text: &'a str) -> Result<Self, Error>;
}

impl<'a> Parsable<'a> for usize {
    fn parse(text: &'a str) -> Result<Self, Error> {
        text.parse().map_err(|_| Error::MalformedArguments)
    }
}

impl<'a> Parsable<'a> for u32 {
    fn parse(text: &'a str) -> Result<Self, Error> {
        text.parse().map_err(|_| Error::MalformedArguments)
    }
}

impl<'a> Parsable<'a> for u8 {
    fn parse(text: &'a str) -> Result<Self, Error> {
        text.parse().map_err(|_| Error::MalformedArguments)
    }
}

impl<'a> Parsable<'a> for &'a str {
    fn parse(text: &'a str) -> Result<Self, Error> { Ok(text) }
}

trait RetrieveArgument<T> {
    fn retrieve(&self, name: &str) -> Result<T, Error>;
}

impl<'a, T: Parsable<'a>> RetrieveArgument<T> for ArgumentIterator<'a> {
    fn retrieve(&self, name: &str) -> Result<T, Error> {
        // At this point we know the argument is a pair, so we error out if it's single
        if self.clone().any(|arg| Argument::Single(name) == arg) {
            return Err(Error::MalformedArguments);
        }

        let argument = self
            .clone()
            .find_map(|arg| match arg {
                Argument::Pair(n, v) if n == name => Some(v),
                _ => None,
            })
            .ok_or(Error::MissingArgument)?;
        T::parse(argument)
    }
}

impl<'a> RetrieveArgument<bool> for ArgumentIterator<'a> {
    fn retrieve(&self, name: &str) -> Result<bool, Error> {
        Ok(self.clone().any(|arg| arg.name() == name))
    }
}

impl<'a, T: Parsable<'a>> RetrieveArgument<Option<T>> for ArgumentIterator<'a> {
    fn retrieve(&self, name: &str) -> Result<Option<T>, Error> {
        // At this point we know the argument is a pair, so we error out if it's single
        if self.clone().any(|arg| Argument::Single(name) == arg) {
            return Err(Error::MalformedArguments);
        }

        let argument = self.clone().find_map(|arg| match arg {
            Argument::Pair(n, v) if n == name => Some(v),
            _ => None,
        });

        if let Some(argument) = argument {
            Ok(Some(T::parse(argument)?))
        } else {
            Ok(None)
        }
    }
}

const ARGUMENT_SEPARATOR: char = '=';
const ALLOWED_TOKENS: &str = " =_";
const LINE_TERMINATOR: char = '\n';

impl<SRL: Serial> Cli<SRL> {
    /// Reads a line, parses it as a command and attempts to execute it.
    pub fn run<MCUF: Flash, EXTF: Flash>(
        &mut self,
        boot_manager: &mut BootManager<MCUF, EXTF, SRL>,
        greeting: &'static str,
    ) {
        if !self.greeted {
            uprintln!(self.serial, "");
            uprintln!(self.serial, "{}", greeting);
            uprintln!(self.serial, "Type `help` for a list of commands");
            self.greeted = true;
        }
        if self.needs_prompt {
            uprint!(self.serial, "{}", PROMPT);
            self.needs_prompt = false;
        }
        let mut execute_command = || -> Result<(), Error> {
            let mut buffer = [0u8; BUFFER_SIZE];
            block!(self.read_line(&mut buffer))?;
            let text = from_utf8(&buffer).map_err(|_| Error::BadCommandEncoding)?;
            let (name, arguments) = Self::parse(text)?;
            commands::run(self, boot_manager, name, arguments)?;
            Ok(())
        };
        match execute_command() {
            Err(Error::BadCommandEncoding) => {
                uwriteln!(self.serial, "[CLI Error] Bad command encoding")
            }
            Err(Error::CharactersNotAllowed) => {
                uwriteln!(self.serial, "[CLI Error] Illegal characters in command")
            }
            Err(Error::MalformedArguments) => {
                uwriteln!(self.serial, "[CLI Error] Malformed command arguments")
            }
            Err(Error::SerialBufferOverflow) => {
                uwriteln!(self.serial, "[CLI Error] Command string too long")
            }
            Err(Error::MissingArgument) => {
                uwriteln!(self.serial, "[CLI Error] Command missing an argument")
            }
            Err(Error::DuplicateArguments) => {
                uwriteln!(self.serial, "[CLI Error] Command contains duplicate arguments")
            }
            Err(Error::ApplicationError(e)) => {
                uwriteln!(self.serial, "[CLI Error] Internal boot manager error: ").ok().unwrap();
                e.report(&mut self.serial);
                Ok(())
            }
            Err(Error::UnexpectedArguments) => {
                uwriteln!(self.serial, "[CLI Error] Command contains an unexpected argument")
            }
            Err(Error::ArgumentOutOfRange) => {
                uwriteln!(self.serial, "[CLI Error] Argument is out of valid range")
            }
            Err(Error::SerialReadError) => uwriteln!(self.serial, "[CLI Error] Serial read failed"),
            Err(Error::CommandUnknown) => uwriteln!(self.serial, "Unknown command"),
            Err(Error::CommandEmpty) => Ok(()),
            Ok(_) => Ok(()),
        }
        .ok()
        .unwrap();
        self.needs_prompt = true;
    }

    /// Returns the serial driver the CLI is using.
    pub fn serial(&mut self) -> &mut SRL { &mut self.serial }

    /// Attempts to parse a given string into a command name and arguments.
    fn parse(text: &str) -> Result<(Name, ArgumentIterator), Error> {
        let text = text.trim_end_matches(|c: char| c.is_ascii_control() || c.is_ascii_whitespace());
        if text.is_empty() {
            return Err(Error::CommandEmpty);
        }
        if !text.chars().all(|c| c.is_ascii_alphanumeric() || ALLOWED_TOKENS.contains(c)) {
            return Err(Error::CharactersNotAllowed);
        }

        let mut tokens = text.split_whitespace();
        // Ensure no bad formatting
        let badly_formatted = tokens.clone().any(|token| {
            token.chars().filter(|c| *c == ARGUMENT_SEPARATOR).count() > 1
                || token.starts_with(ARGUMENT_SEPARATOR)
                || token.ends_with(ARGUMENT_SEPARATOR)
        });

        if badly_formatted {
            return Err(Error::MalformedArguments);
        }
        let name = tokens.next().ok_or(Error::CommandEmpty)?;
        let arguments = ArgumentIterator { tokens };
        let unique = arguments
            .clone()
            .map(|arg| match arg {
                Argument::Pair(n, _) => n,
                Argument::Single(n) => n,
            })
            .all_unique();

        if !unique {
            return Err(Error::DuplicateArguments);
        }

        Ok((name, arguments))
    }

    /// Creates a new CLI using the given serial.
    pub fn new(serial: SRL) -> Result<Self, Error> {
        Ok(Cli { serial, greeted: false, needs_prompt: true })
    }

    fn read_line(&mut self, buffer: &mut [u8]) -> nb::Result<(), Error> {
        let mut bytes = Read::bytes(&mut self.serial).take_while(|element| match element {
            Err(_) => true,
            Ok(b) => *b as char != LINE_TERMINATOR,
        });
        if bytes.try_collect_slice(buffer).map_err(|_| Error::SerialReadError)? < buffer.len() {
            Ok(())
        } else {
            Err(nb::Error::Other(Error::SerialBufferOverflow))
        }
    }

    fn print_help(
        &mut self,
        names: &[&'static str],
        helpstrings: &[(&'static str, &[(&'static str, &'static str)])],
        command: Option<&str>,
    ) {
        if let Some(command) = command {
            if !names.iter().any(|n| n == &command) {
                uprintln!(self.serial, "Requested command doesn't exist.");
                return;
            }
        } else {
            uprintln!(self.serial, "List of available commands:");
        }

        for (name, (help, arguments_help)) in names.iter().zip(helpstrings.iter()) {
            if let Some(command) = command.as_ref() {
                if command != name {
                    continue;
                }
            }

            uprintln!(self.serial, "[{}] - {}", name, help);
            for (argument, range) in arguments_help.iter() {
                uprintln!(self.serial, "    * {} -> {}", argument, range);
            }
        }
    }
}

macro_rules! commands {
    (
        $cli:ident, $boot_manager:ident, $names:ident, $helpstrings:ident [
            $(
                $c:ident[$h:expr]($($a:ident: $t:ty [$r:expr],)*) $command:block,
            )+
        ]
    ) => {
        #[allow(non_upper_case_globals)]
        const $names: &[&'static str] = &[
            $(
                stringify!($c),
            )+
        ];
        #[allow(non_upper_case_globals)]
        const $helpstrings: &[(&'static str, &[(&'static str, &'static str)])] = &[
            $(
                ($h, &[
                     $((stringify!($a), $r),)*
                ]),
            )+
        ];

        #[allow(unreachable_code)]
        pub(super) fn run<MCUF: Flash, EXTF: Flash, SRL: Serial>(
            $cli: &mut Cli<SRL>,
            $boot_manager: &mut BootManager<MCUF, EXTF, SRL>,
            name: Name, arguments: ArgumentIterator) -> Result<(), Error>
        {
            match name {
                $(
                    stringify!($c) => {
                        if arguments.clone().any(|_a| true $(&& _a.name() != stringify!($a))*) {
                            return Err(Error::UnexpectedArguments);
                        }

                        $(
                            let $a: $t  = arguments.retrieve(stringify!($a))?;
                        )*

                        $command
                        Ok(())
                    },
                )+
                _ => Err(Error::CommandUnknown),
            }
        }
    };
}

mod commands;

#[cfg(test)]
mod test {
    use crate::error::Convertible;

    use super::*;
    use blue_hal::hal::doubles::serial::*;

    impl Convertible for SerialStubError {
        fn into(self) -> ApplicationError { ApplicationError::DeviceError("Serial stub failed") }
    }

    #[test]
    fn basic_command_parsing() {
        let sample_command = "my_command an_option=5000 some_flag";
        let (name, mut arguments) = Cli::<SerialStub>::parse(sample_command).unwrap();
        assert_eq!("my_command", name);
        assert_eq!(Argument::Pair("an_option", "5000"), arguments.next().unwrap());
        assert_eq!(Argument::Single("some_flag"), arguments.next().unwrap());

        let sample_command = "command         with_too_much_whitespace   but  still=valid   \n\n";
        let (name, mut arguments) = Cli::<SerialStub>::parse(sample_command).unwrap();
        assert_eq!("command", name);
        assert_eq!(Argument::Single("with_too_much_whitespace"), arguments.next().unwrap());
        assert_eq!(Argument::Single("but"), arguments.next().unwrap());
        assert_eq!(Argument::Pair("still", "valid"), arguments.next().unwrap());
    }

    #[test]
    fn parsing_fails_for_various_bad_commands() {
        let bad_command_no_fields = "";
        assert_eq!(
            Error::CommandEmpty,
            Cli::<SerialStub>::parse(bad_command_no_fields).err().unwrap()
        );

        let bad_command_strange_formatting = "command with=a=strange=argument";
        assert_eq!(
            Error::MalformedArguments,
            Cli::<SerialStub>::parse(bad_command_strange_formatting).err().unwrap()
        );

        let bad_command_characters_not_allowed = "com-mand with? bad+characters";
        assert_eq!(
            Error::CharactersNotAllowed,
            Cli::<SerialStub>::parse(bad_command_characters_not_allowed).err().unwrap()
        );
    }
}
