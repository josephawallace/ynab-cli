#![forbid(unsafe_code)]
//! Typed client for the YNAB v1 API.
//!
//! Every successful operation returns the decoded `data` object; HTTP envelopes
//! and transport types stay inside the crate.
pub mod client;
pub mod models;
pub mod services;
pub use client::{ApiError, Client, ClientBuildError, ClientBuilder, RetryPolicy};
pub use models::*;
