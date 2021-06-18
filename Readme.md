# Bluefruit Secure Bootloader Project

Access the Loadstone builder app
[here](https://absw.github.io/loadstone/loadstone_front/published_app).

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
