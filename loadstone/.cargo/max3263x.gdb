file ../target/thumbv7em-none-eabi/release/loadstone
target extended-remote localhost:3333
monitor init
monitor program ../target/thumbv7em-none-eabi/release/loadstone verify
monitor reset halt
load
