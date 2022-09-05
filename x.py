#!/usr/bin/env python3
#
# Build front-end for loadstone.
# Do `./x.py help` for more information.

import os
import subprocess
from sys import argv, stdin
from typing import Callable, List, Optional

# Path of the release ELF file generated by Cargo.
ELF_OUTPUT_PATH = "loadstone/target/thumbv7em-none-eabi/release/loadstone"

# The location to write the final binary to.
BIN_OUTPUT_PATH = "loadstone.bin"


def run_cargo_command(
    subcommand: str, args: List[str], config: str, features: List[str]
) -> bool:
    environment = os.environ.copy()
    environment["LOADSTONE_CONFIG"] = config.strip()

    features_arg = "--features=" + ",".join(features)

    result = subprocess.run(
        ["cargo", "+nightly", subcommand, "--bin=loadstone", features_arg] + args,
        cwd="./loadstone",
        env=environment,
    )

    return result.returncode == 0


def run_cargo_clean() -> bool:
    result = subprocess.run(["cargo", "clean"], cwd="./loadstone")
    return result.returncode == 0


def run_cargo_build(config: str, features: List[str]) -> bool:
    return run_cargo_command(
        "build", ["--release", "--target=thumbv7em-none-eabi"], config, features
    )


def run_cargo_check(config: str, features: List[str]) -> bool:
    return run_cargo_command("clippy", [], config, features)


def run_cargo_test(config: str, features: List[str]) -> bool:
    return run_cargo_command("test", [], config, features)


def objcopy_to_binary(source: str, destination: str) -> bool:
    result = subprocess.run(["arm-none-eabi-objcopy", source, "-Obinary", destination])
    return result.returncode == 0


def read_file_argument(path: str) -> Optional[str]:
    """If `path` is '-' evaluate to stdin, otherwise read it as a file path."""
    if path == "-":
        return stdin.read()
    try:
        with open(path, "r") as file:
            return file.read()
    except:
        return None


def help_general() -> bool:
    print("Build front-end for loadstone.")
    print()
    print("USAGE")
    print("    ./build.py SUBCOMMAND ...")
    print()
    print("SUBCOMMANDS")
    for name in COMMANDS:
        command = COMMANDS[name]
        print("    " + name.ljust(8) + command.summary)
    return True


def help_specific(topic: str) -> bool:
    command = COMMANDS.get(topic)
    if command == None:
        print("Error: help: unknown command `" + topic + "`")
        return False

    print("./build.py " + topic)
    print(command.description)
    print()
    print("USAGE")
    print("    ./build.py " + topic + " " + command.usage)
    return True


def command_help(args: List[str]) -> bool:
    if len(args) < 3:
        return help_general()
    elif len(args) == 3:
        return help_specific(args[2])
    else:
        print("Error: help: excessive arguments")
        return False


def command_clean(args: List[str]) -> bool:
    if len(args) != 2:
        print("Error: clean: expected no arguments.")
        return False

    if os.path.exists(BIN_OUTPUT_PATH):
        os.remove(BIN_OUTPUT_PATH)

    return run_cargo_clean()


def command_build(args: List[str]) -> bool:
    if len(args) < 3:
        print("Error: build: expected at least 1 argument.")
        return False

    config = read_file_argument(args[2])
    if config == None:
        print("Error: build: failed to read `" + args[2] + "`")
        return False

    if not run_cargo_build(config, args[3:]):
        return False

    if not objcopy_to_binary(ELF_OUTPUT_PATH, BIN_OUTPUT_PATH):
        return False

    print("Loadstone binary copied to `" + BIN_OUTPUT_PATH + "`.")
    return True


def command_check(args: List[str]) -> bool:
    if len(args) < 3:
        print("Error: check: expected at least 1 argument.")
        return False

    config = read_file_argument(args[2])
    if config == None:
        print("Error: check: failed to read `" + args[2] + "`")
        return False

    return run_cargo_check(config, args[3:])


def command_test(args: List[str]) -> bool:
    if len(args) < 3:
        print("Error: test: expected at least 1 argument.")
        return False

    config = read_file_argument(args[2])
    if config == None:
        print("Error: test: failed to read `" + args[2] + "`")
        return False

    return run_cargo_test(config, args[3:])


class Command:
    def __init__(
        self,
        function: Callable[[List[str]], bool],
        summary: str,
        description: str,
        usage: str,
    ):
        self.function = function
        self.summary = summary
        self.description = description
        self.usage = usage


COMMANDS = {
    "help": Command(
        command_help,
        "Print help information",
        "Print general info or help about a specific command",
        "SUBCOMMAND?",
    ),
    "clean": Command(
        command_clean,
        "Clean up generated files",
        "Removes all files generated by Cargo and build.py.",
        "",
    ),
    "build": Command(
        command_build,
        "Build loadstone",
        "Build loadstone using the configuration from the provided file. If the given path is '-' "
        + "use standard input for config.",
        "CONFIG_FILE FEATURES...",
    ),
    "check": Command(
        command_check,
        "Check loadstone for errors",
        "Run `cargo check` using the configuration from the provided file. If the given path is "
        + "'-' use standard input for config.",
        "CONFIG_FILE FEATURES...",
    ),
    "test": Command(
        command_test,
        "Test loadstone",
        "Build and run loadstone's tests using the configuration from the provided file. If the "
        + "given path is '-' use standard input for config.",
        "CONFIG_FILE FEATURES...",
    ),
}

if len(argv) > 1:
    command = COMMANDS.get(argv[1])
    if command != None:
        success = command.function(argv)
        code = 0 if success else 1
        exit(code)

command_help([])
exit(1)
