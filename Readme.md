# Bluefruit Secure Bootloader Project

Access the Loadstone builder app
[here](https://absw.github.io/loadstone/loadstone_front/published_app).

# Architecture

Loadstone is organized in an abstract, generic layer, and a port layer.

Ports exist under `loadstone/src/ports/`, and may be fully manually defined or
depend on code generation. Those that depend on code generation require a
configuration file generated in the `loadstone_front` application.
