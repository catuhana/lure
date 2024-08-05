use clap::Subcommand;
use inquire::{validator::Validation, Password, Text};
use regex::Regex;

use super::Command;

#[derive(Subcommand, Debug)]
pub enum CommandSubcommands {
    /// Generate an example lure configuration file and print it.
    Generate,
    /// Revolt commands for obtaining some configuration options.
    #[command(subcommand)]
    Revolt(RevoltSubcommands),
}

#[derive(Subcommand, Debug)]
pub enum RevoltSubcommands {
    /// Login to Revolt to obtain a new session token.
    GetSessionToken,
}

impl Command for CommandSubcommands {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Generate => {
                print!("{}", include_str!("../../resources/config.example.yaml"));
            }
            Self::Revolt(revolt_subcommand) => match *revolt_subcommand {
                RevoltSubcommands::GetSessionToken => {
                    let email = match Text::new("Revolt e-mail:")
                        .with_placeholder("i@love.cat")
                        .with_validator(|email: &str| {
                            if Regex::new(r#"[^@ \t\r\n]+@[^@ \t\r\n]+\.[^@ \t\r\n]+"#)
                                .expect("e-mail validation regex is somehow invalid now.")
                                .is_match(email)
                            {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid(
                                    "Entered e-mail address is invalid.".into(),
                                ))
                            }
                        })
                        .prompt()
                    {
                        Ok(email) => email,
                        Err(_) => return Ok(()),
                    };
                    let password = match Password::new("Revolt password:")
                        .with_validator(|password: &str| {
                            if password.chars().count() > 0 {
                                Ok(Validation::Valid)
                            } else {
                                Ok(Validation::Invalid("Password cannot be empty.".into()))
                            }
                        })
                        .with_help_message("Your password will NOT be stored and will only be used to obtain a session token.")
                        .without_confirmation()
                        .prompt()
                    {
                        Ok(password) => password,
                        Err(_) => return Ok(()),
                    };

                    dbg!(email, password);
                }
            },
        }

        Ok(())
    }
}
