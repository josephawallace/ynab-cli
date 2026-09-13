#![forbid(unsafe_code)]

use std::{
    io::{self, IsTerminal},
    process::ExitCode,
};

use clap::{Parser, error::ErrorKind};
use secrecy::SecretString;
use serde_json::json;
use ynab_cli::{
    auth::{
        CredentialSource, CredentialStore, Credentials, KeyringStore, resolve_credentials,
        verify_status,
    },
    commands::{AuthCommand, Command, Output},
    error::CliError,
    execute,
    output::render,
};
use ynab_sdk::Client;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match ynab_cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            let _ = error.print();
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("{}", CliError::Invocation(error.to_string()).json());
            return ExitCode::from(2);
        }
    };
    let output = cli.output;
    match run(cli.command, output).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&error, output);
            ExitCode::from(error.exit_code())
        }
    }
}

async fn run(command: Command, output: Output) -> Result<(), CliError> {
    let value = match command {
        Command::Auth(args) => execute_auth(args.command).await?,
        command => {
            let credentials = system_credentials()?;
            let client = Client::new(credentials.token);
            let mut stdin = io::stdin().lock();
            execute::execute(command, &client, &mut stdin).await?
        }
    };
    render(&value, output, &mut io::stdout().lock())
}

async fn execute_auth(command: AuthCommand) -> Result<serde_json::Value, CliError> {
    match command {
        AuthCommand::Store => {
            if !io::stdin().is_terminal() {
                return Err(CliError::Credentials(
                    "auth store requires a TTY; use YNAB_ACCESS_TOKEN for non-interactive runs"
                        .into(),
                ));
            }
            let first = rpassword::prompt_password("YNAB access token: ")?;
            let second = rpassword::prompt_password("Confirm access token: ")?;
            if first.is_empty() || first != second {
                return Err(CliError::Credentials(
                    "tokens were empty or did not match".into(),
                ));
            }
            KeyringStore::new()?.set(&first)?;
            Ok(json!({"stored": true}))
        }
        AuthCommand::Clear => {
            KeyringStore::new()?.clear()?;
            Ok(json!({"cleared": true}))
        }
        AuthCommand::Status => {
            let credentials = system_credentials()?;
            let source = credentials.source;
            verify_status(&Client::new(credentials.token), source).await
        }
    }
}

fn system_credentials() -> Result<Credentials, CliError> {
    let environment = std::env::var("YNAB_ACCESS_TOKEN").ok();
    if let Some(token) = environment.as_ref().filter(|token| !token.is_empty()) {
        return Ok(Credentials {
            token: SecretString::from(token.clone()),
            source: CredentialSource::Environment,
        });
    }
    resolve_credentials(None, &KeyringStore::new()?)
}

fn print_error(error: &CliError, output: Output) {
    match output {
        Output::Json => eprintln!("{}", error.json()),
        Output::Table => eprintln!("{:?}", miette::miette!(error.to_string())),
    }
}
