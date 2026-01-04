use crate::cli::Args;
use crate::config::Client;
use clap::CommandFactory;
use clap_complete::{generate, shells};

pub fn generate_completion_script(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Args::command();

    if let Ok(client) = Client::from_toml("config.toml") {
        let tax_numbers = client.tax_numbers().clone();
        let cred_fields = client.credinform_fields().clone();

        if !tax_numbers.is_empty() {
            let tax_values: Vec<&'static str> = tax_numbers
                .into_iter()
                .map(|s| {
                    let leaked: &'static str = Box::leak(s.into_boxed_str());
                    leaked
                })
                .collect();

            let tax_arg_id = cmd
                .get_arguments()
                .find(|a| matches!(a.get_id().as_str(), "tax-number" | "tax_number"))
                .map(|a| a.get_id().clone());

            if let Some(arg_id) = tax_arg_id {
                cmd = cmd.mut_arg(arg_id, move |arg: clap::Arg| {
                    arg.value_parser(clap::builder::PossibleValuesParser::new(tax_values.clone()))
                });
            }
        }

        if !cred_fields.is_empty() {
            let cred_values: Vec<&'static str> = cred_fields
                .into_iter()
                .map(|s| {
                    let leaked: &'static str = Box::leak(s.into_boxed_str());
                    leaked
                })
                .collect();

            let field_arg_id = cmd
                .get_arguments()
                .find(|a| a.get_id().as_str() == "field")
                .map(|a| a.get_id().clone());

            if let Some(arg_id) = field_arg_id {
                cmd = cmd.mut_arg(arg_id, move |arg: clap::Arg| {
                    arg.value_parser(clap::builder::PossibleValuesParser::new(cred_values.clone()))
                });
            }
        }
    }

    match shell {
        "bash" => generate(shells::Bash, &mut cmd, "scoring", &mut std::io::stdout()),
        "zsh" => generate(shells::Zsh, &mut cmd, "scoring", &mut std::io::stdout()),
        "fish" => generate(shells::Fish, &mut cmd, "scoring", &mut std::io::stdout()),
        "powershell" => generate(
            shells::PowerShell,
            &mut cmd,
            "scoring",
            &mut std::io::stdout(),
        ),
        "elvish" => generate(shells::Elvish, &mut cmd, "scoring", &mut std::io::stdout()),
        _ => return Err("Unsupported shell".into()),
    }

    Ok(())
}
