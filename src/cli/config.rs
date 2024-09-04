use std::sync::LazyLock;

use clap::Subcommand;
use inquire::{
    validator::{Validation, ValueRequiredValidator},
    CustomUserError, Password, Text,
};
use regex::Regex;
use reqwest::StatusCode;
use rive_models::{data::LoginData, mfa::MFAData, session::LoginResponse};
use serde::{de, Deserialize, Deserializer};
use tracing::trace;

use super::Command;

const SUCCESSFUL_LOGIN_RESPONSE_TEMPLATE: &str = r#"
Session token successfully generated. Put this to your configuration file where `revolt: session_token` is.

It should look like this:
session_token: "{SESSION_TOKEN}"

If you used the `revolt-api-url` option to login to another instance, you should also put that to your configuration file where `revolt: api_url` is.

Important note: Session token allows full access to your account! Never share it with anyone and if possible, store it securely.
"#;

static REVOLT_SESSION_FRIENDLY_NAME: LazyLock<String> = LazyLock::new(|| {
    format!(
        "lure on {os}{repo}",
        os = std::env::consts::OS,
        repo = env!("CARGO_PKG_REPOSITORY")
            .strip_prefix("https://")
            .map_or_else(String::default, |stripped_repo| format!(
                " ({stripped_repo})"
            ))
    )
});

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[^@ \t\r\n]+@[^@ \t\r\n]+\.[^@ \t\r\n]+")
        .expect("e-mail validation regex is somehow invalid now.")
});
static TOTP_OR_RECOVERY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^([a-z0-9]{5}-[a-z0-9]{5})|([0-9]{6})$")
        .expect("2FA or recovery code regex is somehow invalid now.")
});
static TOTP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^[0-9]{6}$").expect("TOTP regex is somehow invalid now."));

type StaticInquireValidatorFn =
    Box<dyn Fn(&str) -> Result<Validation, CustomUserError> + Sync + Send>;

static INQUIRE_EMAIL_VALIDATOR: LazyLock<StaticInquireValidatorFn> = LazyLock::new(|| {
    Box::new(|email: &str| {
        if EMAIL_REGEX.is_match(email) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(
                "Entered e-mail address is invalid.".into(),
            ))
        }
    })
});
static INQUIRE_TOTP_OR_REGEX_VALIDATOR: LazyLock<StaticInquireValidatorFn> = LazyLock::new(|| {
    Box::new(|code: &str| {
        if TOTP_OR_RECOVERY_REGEX.is_match(code) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(
                "Entered code is not a valid 2FA or recovery code.".into(),
            ))
        }
    })
});
static INQUIRE_TOTP_VALIDATOR: LazyLock<StaticInquireValidatorFn> = LazyLock::new(|| {
    Box::new(|code: &str| {
        if TOTP_REGEX.is_match(code) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(
                "Entered code is not a valid 2FA code.".into(),
            ))
        }
    })
});

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
    GetSessionToken {
        #[arg(long, default_value = "https://api.revolt.chat")]
        revolt_api_url: String,
    },
}

impl Command for CommandSubcommands {
    // :3
    #[allow(clippy::too_many_lines)]
    async fn run(&self) -> anyhow::Result<()> {
        trace!("`config` subcommand");

        match self {
            Self::Generate => {
                trace!("`config generate` subcommand");
                print!("{}", include_str!("../../resources/config.sample.yaml"));
            }
            Self::Revolt(revolt_subcommand) => match revolt_subcommand {
                RevoltSubcommands::GetSessionToken { revolt_api_url } => {
                    trace!("`config revolt get-session-token` subcommand");

                    let reqwest_client = reqwest::Client::new();

                    let Ok(email) = Text::new("E-mail:")
                        .with_placeholder("i@love.cat")
                        .with_validator(INQUIRE_EMAIL_VALIDATOR.as_ref())
                        .prompt()
                    else {
                        return Ok(());
                    };
                    let Ok(password) = Password::new("Password:")
                        .with_validator(ValueRequiredValidator::default())
                        .with_help_message(
                            "We won't keep your password and only use it to get a session token.",
                        )
                        .without_confirmation()
                        .prompt()
                    else {
                        return Ok(());
                    };

                    let login_response: LoginResponse = reqwest_client
                        .post(format!("{revolt_api_url}/auth/session/login"))
                        .json(&LoginData::Email {
                            email,
                            password,
                            friendly_name: Some(REVOLT_SESSION_FRIENDLY_NAME.to_string()),
                        })
                        .send()
                        .await?
                        .handle_user_friendly_error()
                        .await?
                        .json()
                        .await?;

                    match login_response {
                        LoginResponse::Success(session_token) => {
                            println!(
                                "{}",
                                SUCCESSFUL_LOGIN_RESPONSE_TEMPLATE
                                    .replace("{SESSION_TOKEN}", &session_token.token)
                            );
                        }
                        LoginResponse::MFA {
                            ticket: mfa_ticket,
                            allowed_methods,
                        } => {
                            let mfa_prompt = if allowed_methods.len() > 1 {
                                Text::new("Enter 2FA authentication or recovery code:")
                                    .with_validator(INQUIRE_TOTP_OR_REGEX_VALIDATOR.as_ref())
                            } else {
                                Text::new("Enter 2FA authentication code:")
                                    .with_validator(INQUIRE_TOTP_VALIDATOR.as_ref())
                            };

                            let mfa_data = match mfa_prompt.prompt() {
                                Ok(mfa_code) => {
                                    if TOTP_REGEX.is_match(&mfa_code) {
                                        MFAData::Totp {
                                            totp_code: mfa_code,
                                        }
                                    } else {
                                        MFAData::Recovery {
                                            recovery_code: mfa_code,
                                        }
                                    }
                                }
                                Err(_) => return Ok(()),
                            };

                            let mfa_response: LoginResponse = reqwest_client
                                .post(format!("{revolt_api_url}/auth/session/login"))
                                .json(&LoginData::MFA {
                                    mfa_ticket,
                                    mfa_response: Some(mfa_data),
                                    friendly_name: Some(REVOLT_SESSION_FRIENDLY_NAME.to_string()),
                                })
                                .send()
                                .await?
                                .handle_user_friendly_error()
                                .await?
                                .json()
                                .await?;

                            match mfa_response {
                                LoginResponse::Success(session_token) => {
                                    println!(
                                        "{}",
                                        SUCCESSFUL_LOGIN_RESPONSE_TEMPLATE
                                            .replace("{SESSION_TOKEN}", &session_token.token)
                                    );
                                }
                                LoginResponse::MFA {
                                    ticket: _,
                                    allowed_methods: _,
                                } => unreachable!("MFA after MFA is not supposed to be possible."),
                                LoginResponse::Disabled { user_id: _ } => {
                                    anyhow::bail!("The account is disabled.");
                                }
                            }
                        }
                        LoginResponse::Disabled { user_id: _ } => {
                            anyhow::bail!("The account is disabled.");
                        }
                    }
                }
            },
        }

        Ok(())
    }
}

// Taken from
// https://github.com/authifier/authifier/blob/7615a17e7b62e65fdd1294ad100f7ed3e1503b9f/crates/authifier/src/result.rs
#[derive(Debug)]
pub enum CommonRevoltLoginErrors {
    UnverifiedAccount,
    InvalidCredentials,
    InvalidToken,

    CompromisedPassword,
    ShortPassword,
    Blacklisted,
    LockedOut,
}

impl<'de> Deserialize<'de> for CommonRevoltLoginErrors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        struct Helper {
            #[serde(rename = "type")]
            error_type: String,
        }

        let helper = Helper::deserialize(deserializer)?;
        match helper.error_type.as_str() {
            "UnverifiedAccount" => Ok(Self::UnverifiedAccount),
            "InvalidCredentials" => Ok(Self::InvalidCredentials),
            "InvalidToken" => Ok(Self::InvalidToken),
            "CompromisedPassword" => Ok(Self::CompromisedPassword),
            "ShortPassword" => Ok(Self::ShortPassword),
            "Blacklisted" => Ok(Self::Blacklisted),
            "LockedOut" => Ok(Self::LockedOut),
            _ => Err(de::Error::unknown_variant(
                &helper.error_type,
                &[
                    "UnverifiedAccount",
                    "InvalidCredentials",
                    "InvalidToken",
                    "CompromisedPassword",
                    "ShortPassword",
                    "Blacklisted",
                    "LockedOut",
                ],
            )),
        }
    }
}

trait ReqwestResponseExt: Sized {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self>;
}

impl ReqwestResponseExt for reqwest::Response {
    async fn handle_user_friendly_error(self) -> anyhow::Result<Self> {
        match self.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(self),
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                match self.json::<CommonRevoltLoginErrors>().await? {
                    CommonRevoltLoginErrors::UnverifiedAccount =>
                        anyhow::bail!("The account you are trying to log in to is unverified."),
                    CommonRevoltLoginErrors::InvalidCredentials =>
                        anyhow::bail!("Invalid login credentials provided."),
                    CommonRevoltLoginErrors::InvalidToken =>
                        anyhow::bail!("Incorrect 2FA code provided."),
                    CommonRevoltLoginErrors::CompromisedPassword =>
                        anyhow::bail!("The entered password is compromised. Please ensure you have entered the correct password."),
                    CommonRevoltLoginErrors::ShortPassword =>
                        anyhow::bail!("The entered password is too short. Please ensure you have entered the correct password."),
                    CommonRevoltLoginErrors::Blacklisted =>
                        anyhow::bail!("The entered email is blacklisted. Please ensure you have entered the correct email."),
                    CommonRevoltLoginErrors::LockedOut =>
                        anyhow::bail!("This account is locked out. Please try again some time later.")
                }
            }
            _ => anyhow::bail!("Unexpected error: {}", self.text().await?),
        }
    }
}
