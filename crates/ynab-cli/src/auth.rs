use std::fmt;

use keyring::Entry;
use secrecy::SecretString;
use serde::Serialize;
use serde_json::{Value, json};
use ynab_sdk::Client;

use crate::error::CliError;

pub const KEYRING_SERVICE: &str = "ynab-cli";
pub const KEYRING_ACCOUNT: &str = "access-token";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CredentialSource {
    Environment,
    Keyring,
}

impl fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment => f.write_str("environment"),
            Self::Keyring => f.write_str("keyring"),
        }
    }
}

pub struct Credentials {
    pub token: SecretString,
    pub source: CredentialSource,
}

pub trait CredentialStore {
    fn get(&self) -> Result<Option<String>, CliError>;
    fn set(&self, token: &str) -> Result<(), CliError>;
    fn clear(&self) -> Result<(), CliError>;
}

pub struct KeyringStore {
    entry: Entry,
}

impl KeyringStore {
    pub fn new() -> Result<Self, CliError> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).map_err(|error| {
            CliError::Local(format!(
                "OS keyring is unavailable: {error}; set YNAB_ACCESS_TOKEN instead"
            ))
        })?;
        Ok(Self { entry })
    }
}

impl CredentialStore for KeyringStore {
    fn get(&self) -> Result<Option<String>, CliError> {
        match self.entry.get_password() {
            Ok(value) if value.is_empty() => Ok(None),
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(CliError::Local(format!(
                "OS keyring is unavailable: {error}; set YNAB_ACCESS_TOKEN instead"
            ))),
        }
    }

    fn set(&self, token: &str) -> Result<(), CliError> {
        self.entry.set_password(token).map_err(|error| {
            CliError::Local(format!(
                "OS keyring is unavailable: {error}; set YNAB_ACCESS_TOKEN instead"
            ))
        })
    }

    fn clear(&self) -> Result<(), CliError> {
        match self.entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(CliError::Local(format!(
                "OS keyring is unavailable: {error}; no plaintext fallback was used"
            ))),
        }
    }
}

pub fn resolve_credentials(
    environment_token: Option<String>,
    store: &dyn CredentialStore,
) -> Result<Credentials, CliError> {
    if let Some(token) = environment_token.filter(|value| !value.is_empty()) {
        return Ok(Credentials {
            token: SecretString::from(token),
            source: CredentialSource::Environment,
        });
    }
    if let Some(token) = store.get()? {
        return Ok(Credentials {
            token: SecretString::from(token),
            source: CredentialSource::Keyring,
        });
    }
    Err(CliError::Credentials(
        "missing credentials: set YNAB_ACCESS_TOKEN or use auth store".into(),
    ))
}

pub async fn verify_status(client: &Client, source: CredentialSource) -> Result<Value, CliError> {
    client.user().get().await?;
    Ok(json!({ "source": source }))
}
