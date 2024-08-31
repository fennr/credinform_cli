mod cli;
mod credinform;

use clap::Parser;
use cli::Args;
use credinform::{api, AccessToken, CredinformData};
use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();

    let client: Client = Client::new();

    let token: AccessToken = api::get_token(&client).await?;

    let data: CredinformData =
        api::get_data(&client, &token, &args.tax_number, &args.address).await?;

    data.save_data(&args.address)?;

    Ok(())
}
