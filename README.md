# Loadstone Secure Bootloader

Loadstone is a free and open *secure bootloader* for bare-metal and RTOS
applications developed at [Bluefruit Software](https://www.bluefruit.co.uk/).
It's highly modular in order to enforce a small memory footprint (under
32kb with CRC image validation, and under 64kb with ECDSA image signing), easy
to compile and port to different MCU architectures.

Loadstone rests atop the [blue_hal](https://github.com/absw/blue_hal) crate,
which is a collection of Rust hardware abstractions and drivers developed
at Bluefruit.

A unique feature of Loadstone is [its builder
app](https://absw.github.io/loadstone/loadstone_front/published_app). This
graphical application allows you to define the collection of features and exact
memory layout for your application, then trigger an automated Github Actions
build. No tools or installation required, just navigate the GUI and get your
final binary ready to flash!

# Supported features

Loadstone currently supports:
* Multiple image banks to store, copy, verify and boot firmware images. Image
  banks are fully configurable and flexible.
* Support for an optional external flash chip.
* Golden image rollbacks.
* Automatic or app-triggered updates.
* Image integrity guarantee via CRC check.
* Image integrity and authenticity guarentees via ECDSA P256 signature
  verification (an image signing tool is provided under the `tools/` directory.)
* Serial communication for boot process reporting.
* Serial recovery mode.
* Indirect bootloader-app and app-bootloader communication.
* Companion demo application with a feature-rich CLI to test all Loadstone
  features on target.

These features are modular and some of them may be available only for particular
ports. At the moment, the port with the highest amount of support is the
`stm32f412` family.


# Architecture

Loadstone is organized in an abstract, generic layer, and a port layer.

Ports exist under `loadstone/src/ports/`, and may be fully manually defined or
depend on code generation. Those that depend on code generation require a
configuration file generated in the `loadstone_front` application.

To know more about code generation and when/how to use it when expanding
Loadstone, check out the [documentation section for code
generation.](./documentation/codegen.md)

# Building

Building Loadstone requires embedding configuration in a `LOADSTONE_CONFIG`
environment variable. It can be assigned an empty string, if you're just looking
to run unit tests or to build Loadstone for a board that doesn't require code
generation (one that you've defined manually under `loadstone/src/ports`.

```bash
# Run unit tests
LOADSTONE_CONFIG='' cargo test

# Building a codegen port
LOADSTONE_CONFIG=`cat my_stm32_config.ron` cargo b loadstone --features stm32f412

# Building a manual port
LOADSTONE_CONFIG='' cargo b loadstone --features my_manual_port
```
