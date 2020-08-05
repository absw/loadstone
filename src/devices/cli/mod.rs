#![macro_use]
use crate::{
    devices::bootloader::Bootloader,
    error::Error as BootloaderError,
    hal::{flash, serial},
    utilities::{buffer::TryCollectSlice, iterator::Unique},
};
use core::str::{from_utf8, SplitWhitespace};
use nb::block;
use ufmt::{uwrite, uwriteln};

const GREETING: &str = "--=Lodestone CLI=--\ntype `help` for a list of commands.";
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
    BootloaderError(BootloaderError),
}

impl From<BootloaderError> for Error {
    fn from(e: BootloaderError) -> Self { Error::BootloaderError(e) }
}

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

impl<SRL: serial::ReadWrite> Cli<SRL> {
    pub fn run<EXTF, MCUF>(&mut self, bootloader: &mut Bootloader<EXTF, MCUF, SRL>)
    where
        EXTF: flash::ReadWrite,
        BootloaderError: From<EXTF::Error>,
        MCUF: flash::ReadWrite,
        BootloaderError: From<MCUF::Error>,
        SRL: serial::ReadWrite,
        BootloaderError: From<<SRL as serial::Read>::Error>,
    {
        if !self.greeted {
            uprintln!(self.serial, "{}", GREETING);
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
            commands::run(self, bootloader, name, arguments)?;
            Ok(())
        };
        match execute_command() {
            Err(Error::BadCommandEncoding) => {
                uwriteln!(self.serial, "[CLI Error] Bad Command Encoding")
            }
            Err(Error::CharactersNotAllowed) => {
                uwriteln!(self.serial, "[CLI Error] Illegal Characters In Command")
            }
            Err(Error::MalformedArguments) => {
                uwriteln!(self.serial, "[CLI Error] Malformed Command Arguments")
            }
            Err(Error::SerialBufferOverflow) => {
                uwriteln!(self.serial, "[CLI Error] Command String Too Long")
            }
            Err(Error::MissingArgument) => {
                uwriteln!(self.serial, "[CLI Error] Command Missing An Argument")
            }
            Err(Error::DuplicateArguments) => {
                uwriteln!(self.serial, "[CLI Error] Command Contains Duplicate Arguments")
            }
            Err(Error::BootloaderError(e)) => {
                uprintln!(self.serial, "[CLI Error] Internal Bootloader Error: ");
                Ok(e.report(&mut self.serial))
            }
            Err(Error::UnexpectedArguments) => {
                uwriteln!(self.serial, "[CLI Error] Command Contains An Unexpected Argument")
            }
            Err(Error::ArgumentOutOfRange) => {
                uwriteln!(self.serial, "[CLI Error] Argument Is Out Of Valid Range")
            }
            Err(Error::SerialReadError) => uwriteln!(self.serial, "[CLI Error] Serial Read Failed"),
            Err(Error::CommandUnknown) => uwriteln!(self.serial, "Unknown Command"),
            Err(Error::CommandEmpty) => Ok(()),
            Ok(_) => Ok(()),
        }
        .ok()
        .unwrap();
        self.needs_prompt = true;
    }

    pub fn serial(&mut self) -> &mut SRL { &mut self.serial }

    fn parse<'a>(text: &'a str) -> Result<(Name, ArgumentIterator), Error> {
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
                || token.chars().next() == Some(ARGUMENT_SEPARATOR)
                || token.chars().last() == Some(ARGUMENT_SEPARATOR)
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

    pub fn new(serial: SRL) -> Result<Self, Error> {
        Ok(Cli { serial, greeted: false, needs_prompt: true })
    }

    fn read_line(&mut self, buffer: &mut [u8]) -> nb::Result<(), Error> {
        let mut bytes = self.serial.bytes().take_while(|element| match element {
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
        $cli:ident, $bootloader:ident, $names:ident, $helpstrings:ident [
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
        pub(super) fn run<EXTF, MCUF, SRL>(
            $cli: &mut Cli<SRL>,
            $bootloader: &mut Bootloader<EXTF, MCUF, SRL>,
            name: Name, arguments: ArgumentIterator) -> Result<(), Error>
        where
            EXTF: flash::ReadWrite, BootloaderError: From<EXTF::Error>,
            MCUF: flash::ReadWrite, BootloaderError: From<MCUF::Error>,
            SRL: serial::ReadWrite, BootloaderError: From<<SRL as serial::Read>::Error>,
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
    use super::*;
    use crate::hal::doubles::serial::*;

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
