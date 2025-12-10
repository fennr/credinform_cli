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
      --fns                       Получить данные из API FNS
  -h, --help                     Print help

## Конфигурация

Пример `config.toml`:

```

[credinform]
username = "user@example.com"
password = "secret"
api_version = "1.7"
tax_numbers = ["7838368395"]
fields = ["BasicInformation"]

[fns]
token = "replace-with-your-fns-token"
fields = ["search", "egr", "check", "vyp"]

```

## Режимы работы
- `--full` — выгрузка всех полей Credinform из `fields` по всем ИНН + данные FNS по `fns_fields`.
- `--fns` — запрос только к API FNS (по переданному `--tax-number`).
- Комбинации с `--address` и `--trademarks` продолжают работать как раньше; при добавлении `--fns` выгружается и FNS для указанного ИНН.
```
