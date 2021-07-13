use std::{borrow::Cow, io::{Read, Write}};
use loadstone_config::{Configuration, features::Greetings};

fn main() {
    let mut input = String::default();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("Error: failed to read from standard input stream: {}.", e);
        return;
    }

    let mut configuration = if input == "" {
        Configuration::default()
    } else {
        match ron::from_str(&input) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Error: failed to load configuration from input: {}", e);
                return;
            }
        }
    };

    // TODO: Use command line arguments to modify configuration.
    configuration.feature_configuration.greetings = Greetings::Custom {
        loadstone: Cow::from("Hello"),
        demo: Cow::from("Goodbye"),
    };

    let output = match ron::to_string(&configuration) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: failed to write configuration to output: {}", e);
            return;
        }
    };

    if let Err(e) = std::io::stdout().write_all(output.as_bytes()) {
        eprintln!("Error: failed to write output to standard output stream: {}", e);
        return;
    }
}
