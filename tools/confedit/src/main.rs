use std::{borrow::Cow, io::{Read, Write}};
use clap::clap_app;
use loadstone_config::{Configuration, features::{Greetings, Serial}, memory::Bank, security::{SecurityConfiguration, SecurityMode}};

struct Arguments {
    internal_banks: Option<Vec<u32>>,
    external_banks: Option<Vec<u32>>,
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
    if string.is_empty() {
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

    if let Some(banks) = arguments.internal_banks {
        let mut offset = configuration.memory_configuration.internal_memory_map.bootloader_location
            + (configuration.memory_configuration.internal_memory_map.bootloader_length_kb * 1024);

        println!("{:?}", banks);

        configuration.memory_configuration.internal_memory_map.banks = banks.into_iter()
            .map(|size| {
                let bank = Bank {
                    size_kb: size,
                    start_address: offset,
                };
                offset += size;
                bank
            }).collect();
    }

    if let Some(banks) = arguments.external_banks {
        let mut offset = 0;

        configuration.memory_configuration.external_memory_map.banks = banks.into_iter()
            .map(|size| {
                let bank = Bank {
                    size_kb: size,
                    start_address: offset,
                };
                offset += size;
                bank
            }).collect();
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

fn to_decimal_digit(c: char) -> Option<u32> {
    match c {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        _ => None
    }
}

fn parse_banks(string: &str) -> Result<Vec<u32>, String> {
    let mut sizes = Vec::new();
    let mut size : u32 = 0;
    for c in string.chars() {
        if let Some(d) = to_decimal_digit(c) {
            size = (size * 10) + d;
        } else if c == ',' {
            sizes.push(size);
            size = 0;
        } else {
            return Err(format!("bank size list expects decimal digits and commas, found {}.", c))
        }
    };

    if size > 0 {
        sizes.push(size);
    }

    Ok(sizes)
}

fn run_clap() -> Result<Arguments, String> {
    let matches = clap_app!(app =>
        (name: env!("CARGO_PKG_NAME"))
        (version: env!("CARGO_PKG_VERSION"))
        (@arg greeting: --greeting +takes_value)
        (@arg golden: --golden +takes_value)
        (@arg recovery: --recovery +takes_value)
        (@arg internal_banks: --internalbanks +takes_value)
        (@arg external_banks: --externalbanks +takes_value)
    )
    .get_matches();

    let greeting = matches.value_of("greeting").map(String::from);

    let golden_bank = if let Some(s) = matches.value_of("golden") {
        if s == "none" {
            Some(None)
        } else {
            let n = s.parse::<usize>()
                .map_err(|_| "--golden-bank expected an unsigned integer argument".to_string())?;
            Some(Some(n))
        }
    } else {
        None
    };

    let recovery = match matches.value_of("recovery") {
        None => None,
        Some("true") => Some(true),
        Some("false") => Some(false),
        Some(_) => return Err("--recovery expected a boolean argument".to_string()),
    };

    let internal_banks = match matches.value_of("internal_banks") {
        None => None,
        Some(string) => Some(parse_banks(string)?),
    };

    let external_banks = match matches.value_of("external_banks") {
        None => None,
        Some(string) => Some(parse_banks(string)?),
    };

    Ok(Arguments {
        internal_banks,
        external_banks,
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
