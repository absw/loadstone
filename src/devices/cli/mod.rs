#![macro_use]
use crate::hal::serial;
use core::{iter::Map, str::SplitWhitespace};
use core::str::FromStr;

macro_rules! commands {
    (
        $cli: ident [
            $(
                $c:ident($($a:ident$(: $t:ty)?,)*) $command:block,
            )+
        ]
    ) => {
        pub(super) fn run<S: serial::Write>($cli: &mut Cli<S>, name: Name, arguments: Arguments) {
            match name {
                $(
                    stringify!($c) => {
                        $(
                            // If argument is just a flag, get as bool
                            let $a = arguments.is_set(stringify!($a));
                            // If not a flag, shadow with its value
                            $(let $a: Option<$t> = arguments.get_value(stringify!($a));)?
                        )*
                        $command
                    },
                )+
                _ => (),
            }
        }
    };
}

mod commands;

const GREETING: &str = "--=Lodestone CLI=--";

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    CommandEmpty,
    CommandUnknown,
    MalformedArguments,
    CharactersNotAllowed,
}

pub struct Cli<S: serial::Write> {
    serial: S,
}

type Name<'a> = &'a str;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Argument<'a> {
    Single(&'a str),
    Pair(&'a str, &'a str),
}

#[derive(Clone)]
struct Arguments<'a> {
    tokens: SplitWhitespace<'a>,
}

impl<'a> Iterator for Arguments<'a> {
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

impl<'a> Arguments<'a> {
    fn is_set(&self, name: &str) -> bool {
        self.clone().any(|arg| {
            match arg {
                Argument::Pair(n, _) => n == name,
                Argument::Single(n) => n == name,
            }
        })
    }

    /// Locates a specific pair-argument value given its name and type
    fn get_value<T: FromStr>(&self, name: &str) -> Option<T> {
        self.clone().find_map(|arg| {
            match arg {
                Argument::Pair(n, v) if n == name => { n.parse().ok() },
                _ => None,
            }
        })
    }
}

const ARGUMENT_SEPARATOR: char = '=';
const ALLOWED_TOKENS: &str = " =_";

impl<S: serial::Write> Cli<S> {
    fn parse<'a>(text: &'a str) -> Result<(Name, Arguments), Error> {
        let text = text.trim_end_matches(|c: char| "\r\n ".contains(c));
        if !text.chars().all(|c| c.is_alphanumeric() || ALLOWED_TOKENS.contains(c)) {
            return Err(Error::CharactersNotAllowed);
        }

        let mut tokens = text.split_whitespace();
        let name = tokens.next().ok_or(Error::CommandEmpty)?;
        Ok((name, Arguments { tokens }))
    }

    fn new(serial: S) -> Result<Self, Error> {
        unimplemented!();
    }

    fn interpret_line(&mut self, line: &str) -> Result<(), Error> {
        let (name, arguments) = Self::parse(line)?;
        commands::run(self, name, arguments);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    //    #[test]
    //    fn basic_command_parsing() {
    //        let sample_command = "my_command an_option=5000 some_flag";
    //        let (name, mut arguments) = Cli::parse(sample_command).unwrap();
    //        assert_eq!("my_command", name);
    //        assert_eq!(Argument::Pair("an_option", "5000"), arguments.next().unwrap());
    //        assert_eq!(Argument::Single("some_flag"), arguments.next().unwrap());
    //
    //        let sample_command =
    //            "command         with_too_much_whitespace   but  still=valid   \r\n\r\n";
    //        let (name, mut arguments) = Cli::parse(sample_command).unwrap();
    //        assert_eq!("command", name);
    //        assert_eq!(Argument::Single("with_too_much_whitespace"), arguments.next().unwrap());
    //        assert_eq!(Argument::Single("but"), arguments.next().unwrap());
    //        assert_eq!(Argument::Pair("still", "valid"), arguments.next().unwrap());
    //    }
    //
    //    #[test]
    //    fn parsing_fails_for_various_bad_commands() {
    //        let bad_command_no_fields = "";
    //        assert_eq!(Error::CommandEmpty, Cli::parse(bad_command_no_fields).err().unwrap());
    //
    //        let bad_command_strange_formatting = "command with=a=strange=argument";
    //        assert_eq!(
    //            Error::MalformedArguments,
    //            Cli::parse(bad_command_strange_formatting).err().unwrap()
    //        );
    //
    //        let bad_command_characters_not_allowed = "com-mand with? bad+characters";
    //        assert_eq!(
    //            Error::CharactersNotAllowed,
    //            Cli::parse(bad_command_characters_not_allowed).err().unwrap()
    //        );
    //    }
}
