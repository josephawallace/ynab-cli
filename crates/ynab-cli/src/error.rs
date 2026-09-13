use std::{fmt, io};

use serde_json::{Value, json};
use ynab_sdk::ApiError;

#[derive(Debug)]
pub enum CliError {
    Invocation(String),
    Credentials(String),
    Api(ApiError),
    Local(String),
    Io(io::Error),
    Input(String),
}

impl CliError {
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Invocation(_) => 2,
            Self::Credentials(_) => 3,
            Self::Api(ApiError::Response { .. }) => 4,
            Self::Api(_) | Self::Local(_) | Self::Io(_) | Self::Input(_) => 1,
        }
    }

    #[must_use]
    pub fn json(&self) -> Value {
        let kind = match self {
            Self::Invocation(_) => "invocation",
            Self::Credentials(_) => "credentials",
            Self::Api(ApiError::Response { .. }) => "api",
            Self::Api(_) => "client",
            Self::Local(_) => "local",
            Self::Io(_) => "io",
            Self::Input(_) => "input",
        };
        match self {
            Self::Api(ApiError::Response {
                status,
                id,
                name,
                detail,
                retry_after,
            }) => json!({"error": {
                "kind": kind,
                "status": status.as_u16(),
                "id": id,
                "name": name,
                "detail": detail,
                "retry_after_seconds": retry_after.map(|duration| duration.as_secs()),
            }}),
            _ => json!({"error": {"kind": kind, "message": self.to_string()}}),
        }
    }
}

impl From<ApiError> for CliError {
    fn from(value: ApiError) -> Self {
        Self::Api(value)
    }
}

impl From<io::Error> for CliError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invocation(value)
            | Self::Credentials(value)
            | Self::Local(value)
            | Self::Input(value) => f.write_str(value),
            Self::Api(value) => value.fmt(f),
            Self::Io(value) => value.fmt(f),
        }
    }
}

impl std::error::Error for CliError {}
