use super::credinform::{Address, TaxNumber};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub address: Address,
    #[arg(short, long, default_value = "7838368395")]
    pub tax_number: TaxNumber,
    #[arg(short, long, default_value = "config.toml")]
    pub path: String,
}
