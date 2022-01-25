#!/bin/sh

set -x

[ -z "$GDB" ] && which gdb-multiarch >/dev/null && GDB=gdb-multiarch
[ -z "$GDB" ] && which arm-none-eabi-gdb >/dev/null && GDB=arm-none-eabi-gdb
[ -z "$GDB" ] && ( echo "No usable GDB found >&2"; exit 1; )

"${GDB}" \
    -ex "target extended-remote :3333" \
    -ex "monitor init" \
    -ex "monitor program ./node_brain verify" \
    -ex "monitor reset halt" \
    -ex "load" \
    "$@"
