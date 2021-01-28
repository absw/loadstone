#!/bin/bash

truncate -s 32768 loadstone.bin
cat loadstone.bin demo_app.bin > combined.bin
