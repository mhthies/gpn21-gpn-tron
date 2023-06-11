# GPN-Tron bot

This is my [gpn-tron](https://github.com/freehuntx/gpn-tron) bot implementation from Gulasch-Programmier-Nacht 2023 (GPN 21), known from the *michael-1.0*, *michael-1.0.2* and *michael-1.0.3* bots.

## Running

Compile and run with
```bash
cargo run
```

The program accepts one command line argument with the path of the config file.
If not given, a `config.toml` in the current working directory is expected.
An example config file is given in [config.example.toml](config.example.toml).

To enable logging, set the `RUST_LOG` environment variable to the desired log level (e.g. `RUST_LOG=info`).
