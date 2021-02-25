# Loadstone webserver

Webserver using HTTP and WebSocket for communicating with the demo_app over wifi.

## Building

Requires [Rust](https://www.rust-lang.org/) and [Elm](https://elm-lang.org/) to be installed.

Do `./build_web_assets` to build all web files into the `public_html/` directory.

Do `cargo build --release` to build the server executable.

## Running

Do `cargo run --release <path>` to run the server. This requires the web assets to be built (so it can serve web files properly). `<path>` should be replaced with the path to the device with loadstone on.
