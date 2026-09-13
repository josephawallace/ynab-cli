use std::{cell::Cell, io::Cursor};

use clap::Parser;
use secrecy::SecretString;
use serde_json::json;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header_exists, method, path, query_param},
};
use ynab_cli::{
    Cli,
    auth::{CredentialSource, CredentialStore, resolve_credentials, verify_status},
    commands::Output,
    error::CliError,
    execute,
    output::render,
};
use ynab_sdk::Client;

const PLAN: &str = "00000000-0000-0000-0000-000000000001";
const ID: &str = "00000000-0000-0000-0000-000000000002";

fn client(server: &MockServer) -> Client {
    Client::builder(SecretString::from("synthetic-cli-secret"))
        .base_url(Url::parse(&server.uri()).expect("wiremock URL"))
        .build()
        .expect("client")
}

#[tokio::test]
async fn payee_create_emits_resource_and_wraps_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/plans/{PLAN}/payees")))
        .and(header_exists("authorization"))
        .and(body_json(json!({"payee":{"name":"Acme"}})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"data":{
            "payee":{"id":ID,"name":"Acme","transfer_account_id":null,"deleted":false},
            "server_knowledge":2
        }})))
        .expect(1)
        .mount(&server)
        .await;
    let cli = Cli::try_parse_from([
        "ynab-cli", "payee", "create", "--plan", PLAN, "--name", "Acme",
    ])
    .expect("valid CLI");
    let value = execute::execute(cli.command, &client(&server), &mut Cursor::new([]))
        .await
        .expect("execution succeeds");
    assert_eq!(value["id"], ID);
    assert_eq!(value["name"], "Acme");
}

#[tokio::test]
async fn category_transaction_list_selects_one_documented_route() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/plans/{PLAN}/categories/{ID}/transactions")))
        .and(header_exists("authorization"))
        .and(query_param("since_date", "2026-01-01"))
        .and(query_param("until_date", "2026-01-31"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data":{"transactions":[],"server_knowledge":3}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let cli = Cli::try_parse_from([
        "ynab-cli",
        "transaction",
        "list",
        "--plan",
        PLAN,
        "--category",
        ID,
        "--since",
        "2026-01-01",
        "--until",
        "2026-01-31",
    ])
    .expect("valid CLI");
    let value = execute::execute(cli.command, &client(&server), &mut Cursor::new([]))
        .await
        .expect("execution succeeds");
    assert_eq!(value["transactions"], json!([]));
}

#[tokio::test]
async fn invalid_stdin_input_stops_before_http() {
    let server = MockServer::start().await;
    let cli = Cli::try_parse_from([
        "ynab-cli",
        "transaction",
        "create",
        "--plan",
        PLAN,
        "--input",
        "-",
    ])
    .expect("valid CLI");
    let error = execute::execute(cli.command, &client(&server), &mut Cursor::new(br"{}"))
        .await
        .expect_err("empty wrapper is invalid");
    assert!(matches!(error, CliError::Input(_)));
    assert!(
        server
            .received_requests()
            .await
            .expect("requests")
            .is_empty()
    );
}

struct FakeStore {
    value: Result<Option<String>, &'static str>,
    reads: Cell<usize>,
}

impl CredentialStore for FakeStore {
    fn get(&self) -> Result<Option<String>, CliError> {
        self.reads.set(self.reads.get() + 1);
        self.value
            .clone()
            .map_err(|message| CliError::Local(message.into()))
    }

    fn set(&self, _token: &str) -> Result<(), CliError> {
        Ok(())
    }

    fn clear(&self) -> Result<(), CliError> {
        Ok(())
    }
}

#[test]
fn environment_credential_precedes_keyring() {
    let store = FakeStore {
        value: Ok(Some("keyring-secret".into())),
        reads: Cell::new(0),
    };
    let credentials = resolve_credentials(Some("environment-secret".into()), &store)
        .expect("environment credential");
    assert_eq!(credentials.source, CredentialSource::Environment);
    assert_eq!(store.reads.get(), 0);
}

#[test]
fn unavailable_keyring_is_local_failure() {
    let store = FakeStore {
        value: Err("keyring unavailable"),
        reads: Cell::new(0),
    };
    let error = resolve_credentials(None, &store)
        .err()
        .expect("keyring failure");
    assert!(matches!(error, CliError::Local(_)));
}

#[test]
fn missing_credential_is_exit_three() {
    let store = FakeStore {
        value: Ok(None),
        reads: Cell::new(0),
    };
    let error = resolve_credentials(None, &store)
        .err()
        .expect("missing credential");
    assert_eq!(error.exit_code(), 3);
}

#[tokio::test]
async fn auth_status_verifies_the_token_with_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header_exists("authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "user": { "id": ID } }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let value = verify_status(&client(&server), CredentialSource::Environment)
        .await
        .expect("valid credentials");
    assert_eq!(value, json!({ "source": "environment" }));
}

#[test]
fn json_errors_are_structured() {
    let error = CliError::Credentials("missing".into());
    assert_eq!(error.json()["error"]["kind"], "credentials");
    assert_eq!(error.json()["error"]["message"], "missing");
}

#[test]
fn table_renderer_is_selected_without_changing_value() {
    let mut output = Vec::new();
    render(
        &json!([{"id":ID,"name":"Acme"}]),
        Output::Table,
        &mut output,
    )
    .expect("render succeeds");
    let text = String::from_utf8(output).expect("UTF-8");
    assert!(text.contains("Acme"));
    assert!(text.contains("name"));
}

#[test]
fn table_renderer_extracts_collection_rows() {
    let mut output = Vec::new();
    render(
        &json!({"accounts": [{"id": ID, "name": "Checking", "balance": 1000}], "server_knowledge": 3}),
        Output::Table,
        &mut output,
    )
    .expect("render succeeds");
    let text = String::from_utf8(output).expect("UTF-8");
    assert!(text.contains("Checking"));
    assert!(text.contains("balance"));
    assert!(!text.contains("server_knowledge"));
}
