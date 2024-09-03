mod cli;
mod config;
mod credinform;

use clap::Parser;
use cli::Args;
use config::Client;
use credinform::{api, AccessToken, Address, CredinformData};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();

    let client: Client = Client::from_toml(args.config.as_str())?;

    let token: AccessToken = api::get_token(&client).await?;

    match args {
        Args { full: true, .. } => {
            for address in Address::from_vec(&client.data.credinform.fields) {
                let data: CredinformData =
                    api::get_data(&client, &token, &args.tax_number, &address).await?;
                data.to_file(&address, &args.tax_number)?;
            }
        }
        Args {
            address: Some(address),
            ..
        } => {
            let data: CredinformData =
                api::get_data(&client, &token, &args.tax_number, &address).await?;

            data.to_file(&address, &args.tax_number)?;
        }
        _ => { 
            Err(format!("Invalid arguments, Add '--full' or '--address' to your command"))?
        }
    }

    Ok(())
}
