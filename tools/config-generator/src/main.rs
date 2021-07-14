use std::{borrow::Cow, io::{Read, Write}};
use clap::clap_app;
use loadstone_config::{Configuration, features::{Greetings, Serial}, security::{SecurityConfiguration, SecurityMode}};

struct Arguments {
    greeting: Option<String>,
    golden_bank: Option<Option<usize>>,
    recovery: Option<bool>,
}

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
        Ok(Configuration {
            security_configuration: SecurityConfiguration {
                security_mode: SecurityMode::Crc,
                ..SecurityConfiguration::default()
            },
            ..Configuration::default()
        })
    } else {
        ron::from_str(&string)
            .map_err(|e| format!("failed to load configuration from input: {}.", e))
    }
}

fn modify_configuration(mut configuration: Configuration, arguments: Arguments) -> Result<Configuration, String> {
    if let Some(greeting) = arguments.greeting {
        let old_demo = match configuration.feature_configuration.greetings {
            Greetings::Default => Cow::from(""),
            Greetings::Custom { demo, .. } => demo,
        };

        configuration.feature_configuration.greetings = Greetings::Custom {
            loadstone: Cow::from(greeting),
            demo: old_demo,
        };
    }

    if let Some(bank) = arguments.golden_bank {
        configuration.memory_configuration.golden_index = bank;
    }

    if let Some(recovery) = arguments.recovery {
        let serial = &mut configuration.feature_configuration.serial;
        if let Serial::Enabled { recovery_enabled, .. } = serial {
            *recovery_enabled = recovery;
        } else {
            return Err(String::from("cannot enable serial recovery since serial is not enabled"));
        }
    }

    Ok(configuration)
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
    let arguments = run_clap()?;
    let input = read_input_string()?;
    let configuration = get_input_configuration(input)?;
    let new_configuration = modify_configuration(configuration, arguments)?;
    let output = get_output_string(new_configuration)?;
    write_output_string(output)
}

fn run_clap() -> Result<Arguments, String> {
    let matches = clap_app!(app =>
        (name: env!("CARGO_PKG_NAME"))
        (version: env!("CARGO_PKG_VERSION"))
        (@arg greeting: --greeting +takes_value)
        (@arg golden: --golden +takes_value)
        (@arg recovery: --recovery +takes_value)
    )
    .get_matches();

    let greeting = matches.value_of("greeting").map(String::from);

    let golden_bank = if let Some(s) = matches.value_of("golden") {
        if s == "none" {
            Some(None)
        } else {
            let n = s.parse::<usize>()
                .map_err(|_| format!("--golden-bank expected an unsigned integer argument"))?;
            Some(Some(n))
        }
    } else {
        None
    };

    let recovery = match matches.value_of("recovery") {
        None => None,
        Some("true") => Some(true),
        Some("false") => Some(false),
        Some(_) => Err(format!("--recovery expected a boolean argument"))?,
    };

    Ok(Arguments {
        greeting,
        golden_bank,
        recovery,
    })
}

fn main() {
    let result = run();

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1)
    }
}
