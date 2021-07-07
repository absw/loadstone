#!/bin/sh

# Wrapper that decides which gdb to run (arm-none-eabi-gdb or gdb-multiarch,
# whichever is present), and adds arguments that replace .gdbinit (because a
# lying around .gdbinit is usually not trusted by gdb).
#
# The script is solely suitable for being called as the runner set in
# .cargo/config.
#
# The GDB setup assumes that an OpenOCD is ready and running.

set -x

[ -z "$GDB" ] && which gdb-multiarch >/dev/null && GDB=gdb-multiarch
[ -z "$GDB" ] && which arm-none-eabi-gdb >/dev/null && GDB=arm-none-eabi-gdb
[ -z "$GDB" ] && ( echo "No usable GDB found >&2"; exit 1; )

"${GDB}" \
    -ex "target extended-remote :3333" \
    -ex "svd_load svd/EFM32GG11B820F2048GM64.svd" \
    -ex "set print asm-demangle on" \
    -ex "monitor arm semihosting enable" \
    -ex "load" \
    -ex "monitor reset halt" \
    -ex "continue" \
    "$@"
