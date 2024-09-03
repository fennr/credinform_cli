use super::credinform::{Address, TaxNumber};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(short, long, conflicts_with = "full", help = "Имя ручки в credinform")]
    pub address: Option<Address>,

    #[arg(
        short,
        long,
        conflicts_with = "address",
        default_value = "false",
        help = "Выгрузить все ручки описанные в config.toml"
    )]
    pub full: bool,

    #[arg(short, long, default_value = "7838368395", help = "ИНН компании")]
    pub tax_number: TaxNumber,

    #[arg(
        short,
        long,
        default_value = "config.toml",
        help = "Путь к config.toml"
    )]
    pub config: String,
}
