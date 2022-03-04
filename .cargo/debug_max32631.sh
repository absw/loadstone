#!/bin/sh

set -x

[ -z "$GDB" ] && which gdb-multiarch >/dev/null && GDB=gdb-multiarch
[ -z "$GDB" ] && which arm-none-eabi-gdb >/dev/null && GDB=arm-none-eabi-gdb
[ -z "$GDB" ] && ( echo "No usable GDB found >&2"; exit 1; )

"${GDB}" \
    -ex "target extended-remote :3333" \
    -ex "monitor init" \
    -ex "monitor program target/thumbv7em-none-eabi/release/loadstone verify" \
    -ex "monitor arm semihosting enable" \
    -ex "monitor rtt server start 8765 0" \
    -ex "monitor rtt setup 0x20000000 0x30 \"SEGGER RTT\"" \
    -ex "monitor rtt start" \
    -ex "monitor reset halt" \
    -ex "load" \
    "$@"
