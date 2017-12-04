# Mjolnir Plugin Tester

# Installation

The easiest way to install mjolnir-plugin-tester is to run `cargo install --git=https://github.com/CentauriSolutions/mjolnir-plugin-tester.git mjolnir-plugin-tester` on a machine with the rust toolchain installed.

## Usage

The plugin tester can be configured as shows in [the examples](examples/config.toml). It should then be run as `mjolnir-plugin-tester --config=/path/to/config.toml` and will then validate that the plugin can be registered, instantiated, and run correctly with an arbitrary alert.

It can also be called with the `-q`, or `--quick` option to only validate the registration., Please note that sending the `-q` will _not_ cause the plugin to actually be called, and should only be used for very basic validation. If you want to ensure that your plugin will actually _do_ something with the alert, the tester should be run without additional arguments.
