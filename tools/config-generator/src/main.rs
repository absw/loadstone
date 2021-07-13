use std::{borrow::Cow, io::{Read, Write}};
use loadstone_config::{Configuration, features::Greetings};

fn read_input_string() -> Result<String, String> {
    let mut input = String::default();
    let result = std::io::stdin().read_to_string(&mut input);
    match result {
        Ok(_) => Ok(input),
        Err(e) => Err(format!("failed to read from standard input stream: {}.", e)),
    }
}

fn get_input_configuration(string: String) -> Result<Configuration, String> {
    if string == "" {
        Ok(Default::default())
    } else {
        ron::from_str(&string)
            .map_err(|e| format!("failed to load configuration from input: {}.", e))
    }
}

fn modify_configuration(mut configuration: Configuration) -> Configuration {
    // TODO: Use command line arguments to modify configuration.
    configuration.feature_configuration.greetings = Greetings::Custom {
        loadstone: Cow::from("Hello"),
        demo: Cow::from("Goodbye"),
    };
    configuration
}

fn get_output_string(configuration: Configuration) -> Result<String, String> {
    ron::to_string(&configuration)
        .map_err(|e| format!("failed to write configuration to output: {}.", e))
}

fn write_output_string(string: String) -> Result<(), String> {
    std::io::stdout().write_all(string.as_bytes())
        .map_err(|e| format!("failed to write output to standard output stream: {}.", e))
}

fn run() -> Result<(), String> {
    let input = read_input_string()?;
    let configuration = get_input_configuration(input)?;
    let new_configuration = modify_configuration(configuration);
    let output = get_output_string(new_configuration)?;
    write_output_string(output)
}

fn main() {
    let result = run();

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1)
    }
}
