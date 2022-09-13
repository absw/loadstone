# Using x.py

The intended way to build loadstone is using the x.py script in the root of the project. For usage information do `./x.py help SUBCOMMAND`.

To generate bootloader configurations, build and execute the `loadstone_front` tool in `tools/loadstone_front`. The produced config file can be passed into x.py for a build. (eg. `./x.py build tools/loadstone_front/loadstone_config.ron stm32f412`)

Testing and linting can also be performed this way.
