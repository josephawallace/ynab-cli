use std::str::FromStr;

use secrecy::SecretString;
use ynab_sdk::{AccountType, Client, ClientBuilder, Milliunits, PlanMonth};

#[test]
fn unknown_open_value_round_trips() {
    let value: AccountType = serde_json::from_str("\"future-server-value\"").expect("valid string");
    assert_eq!(value.0, "future-server-value");
    assert_eq!(
        serde_json::to_string(&value).expect("serializes"),
        "\"future-server-value\""
    );
}

#[test]
fn client_debug_redacts_bearer_token() {
    let client = Client::new(SecretString::from("synthetic-secret-value"));
    assert!(!format!("{client:?}").contains("synthetic-secret-value"));
}

#[test]
fn rate_limiter_rejects_documented_ceiling_violation() {
    assert!(
        ClientBuilder::new(SecretString::from("synthetic-secret-value"))
            .requests_per_hour(201)
            .is_err()
    );
}

#[test]
fn milliunits_keep_negative_three_decimal_boundary() {
    let amount = rust_decimal::Decimal::from_str("-0.001").expect("decimal");
    assert_eq!(Milliunits::from_decimal(amount), Some(Milliunits(-1)));
}

#[test]
fn months_serialize_as_first_day() {
    let month = PlanMonth::from_str("2026-09-01").expect("first day");
    assert_eq!(
        serde_json::to_string(&month).expect("serializes"),
        "\"2026-09-01\""
    );
}
