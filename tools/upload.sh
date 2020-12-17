#!/bin/bash

if [ "$#" -ne 2 ]; then
   echo "Usage: $0 <filename> <serial> (example: ./upload.sh image.bin /dev/ttyUSB0)"
   exit 1
fi

sudo stty -F /dev/ttyUSB0 9600 cs8 -parenb -cstopb -ixoff
sx $1 < $2 > $2
