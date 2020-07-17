//type ArgumentList = &'static [ArgumentDescriptor];
type Function = &'static dyn Fn(&str, &mut Cli) -> Result<(), Error>;

pub struct CommandDescriptor {
    name: &'static str,
    function: Function,
}

const COMMANDS: [CommandDescriptor; 1] = [
    CommandDescriptor { name: "help", function: &|_, _| Ok(()) },
];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    CommandEmpty,
    CommandUnknown,
    MalformedArguments,
    CharactersNotAllowed,
}

pub struct Cli { }

type Name<'a> = &'a str;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Argument<'a> {
    Single(&'a str),
    Pair(&'a str, &'a str),
}

const ARGUMENT_SEPARATOR: char = '=';
const ALLOWED_TOKENS: &str = " =_";

impl Cli {
    fn parse<'a>(text: &'a str) -> Result<(Name, impl Iterator<Item=Argument<'a>>), Error> {
        let text = text.trim_end_matches(|c:char| c == '\r' || c == '\n' || c.is_whitespace());
        if !text.chars().all(|c| c.is_alphanumeric() || ALLOWED_TOKENS.contains(c)) {
            return Err(Error::CharactersNotAllowed)
        }

        let mut tokens = text.split_whitespace();
        let name = tokens.next().ok_or(Error::CommandEmpty)?;
        if !tokens.clone().all(|t| t.split(ARGUMENT_SEPARATOR).count() <= 2) {
            return Err(Error::MalformedArguments);
        }
        let arguments = tokens.map(|t| {
            let mut split = t.split(ARGUMENT_SEPARATOR);
            match split.clone().count() {
                2 => Argument::Pair(split.next().unwrap(), split.next().unwrap()),
                _ => Argument::Single(split.next().unwrap()),
            }
        });
        Ok((name, arguments))
    }

    fn interpret_line(&mut self, line: &str) -> Result<(), Error> {
        unimplemented!();
        //let name = Self::tokens(line).nth(0).ok_or(Error::CommandEmpty)?;
        //let command = COMMANDS.iter().find(|c| c.name == name).ok_or(Error::CommandUnknown)?;
        //(command.function)(line, self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_command_parsing() {
        let sample_command = "my_command an_option=5000 some_flag";
        let (name, mut arguments) = Cli::parse(sample_command).unwrap();
        assert_eq!("my_command", name);
        assert_eq!(Argument::Pair("an_option","5000"), arguments.next().unwrap());
        assert_eq!(Argument::Single("some_flag"), arguments.next().unwrap());

        let sample_command = "command         with_too_much_whitespace   but  still=valid   \r\n\r\n";
        let (name, mut arguments) = Cli::parse(sample_command).unwrap();
        assert_eq!("command", name);
        assert_eq!(Argument::Single("with_too_much_whitespace"), arguments.next().unwrap());
        assert_eq!(Argument::Single("but"), arguments.next().unwrap());
        assert_eq!(Argument::Pair("still", "valid"), arguments.next().unwrap());
    }

    #[test]
    fn parsing_fails_for_various_bad_commands() {
        let bad_command_no_fields = "";
        assert_eq!(Error::CommandEmpty, Cli::parse(bad_command_no_fields).err().unwrap());

        let bad_command_strange_formatting = "command with=a=strange=argument";
        assert_eq!(Error::MalformedArguments, Cli::parse(bad_command_strange_formatting).err().unwrap());

        let bad_command_characters_not_allowed = "com-mand with? bad+characters";
        assert_eq!(Error::CharactersNotAllowed, Cli::parse(bad_command_characters_not_allowed).err().unwrap());
    }
}
