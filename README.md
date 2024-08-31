# Credinform API Client

This is a simple client for [Credinform](https://credinform.ru/) API.

## Installation

1. Clone the repository: `git clone https://github.com/fennr/credinform-cli`
2. Install dependencies: `cargo build`
3. Set environment variables `CREDINFORM_USERNAME` and `CREDINFORM_PASSWORD` to your Credinform API credentials
4. Run the client: `cargo run -- [address] [tax number]`

## Usage
```
Usage: credinform_cli.exe [OPTIONS] --address <ADDRESS>

Options:
  -a, --address <ADDRESS>
  -t, --tax-number <TAX_NUMBER>  [default: 7838368395]
  -h, --help                     Print help
```