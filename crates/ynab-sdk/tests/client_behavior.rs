use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use reqwest::StatusCode;
use secrecy::SecretString;
use serde_json::json;
use url::Url;
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{body_json, method, path},
};
use ynab_sdk::{ApiError, Client, PostPayee, RetryPolicy};

#[derive(Clone)]
struct SequenceResponder {
    calls: Arc<AtomicUsize>,
    first: ResponseTemplate,
    next: ResponseTemplate,
}

impl Respond for SequenceResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            self.first.clone()
        } else {
            self.next.clone()
        }
    }
}

fn client(server: &MockServer, retry: RetryPolicy) -> Client {
    Client::builder(SecretString::from("fixture-token".to_owned()))
        .base_url(Url::parse(&format!("{}/v1", server.uri())).unwrap())
        .retry_policy(retry)
        .build()
        .unwrap()
}

fn error_response(status: u16) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({
        "error": {
            "id": "error-id",
            "name": "fixture_error",
            "detail": "fixture detail"
        }
    }))
}

#[tokio::test]
async fn all_documented_error_statuses_are_structured() {
    for status in [400, 401, 403, 404, 409, 429, 500, 503] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/user"))
            .respond_with(error_response(status).insert_header("Retry-After", "2"))
            .expect(1)
            .mount(&server)
            .await;
        let error = client(
            &server,
            RetryPolicy {
                attempts: 1,
                max_backoff: Duration::ZERO,
            },
        )
        .user()
        .get()
        .await
        .unwrap_err();
        match error {
            ApiError::Response {
                status: actual,
                id,
                name,
                detail,
                retry_after,
            } => {
                assert_eq!(actual, StatusCode::from_u16(status).unwrap());
                assert_eq!(id.as_deref(), Some("error-id"));
                assert_eq!(name, "fixture_error");
                assert_eq!(detail, "fixture detail");
                assert_eq!(retry_after, Some(Duration::from_secs(2)));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[tokio::test]
async fn get_retries_429_after_numeric_retry_after() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path("/v1/user"))
        .respond_with(SequenceResponder {
            calls: Arc::clone(&calls),
            first: error_response(429).insert_header("Retry-After", "1"),
            next: ResponseTemplate::new(200).set_body_json(
                json!({ "data": { "user": { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" } } }),
            ),
        })
        .expect(2)
        .mount(&server)
        .await;

    let started = Instant::now();
    client(
        &server,
        RetryPolicy {
            attempts: 2,
            max_backoff: Duration::ZERO,
        },
    )
    .user()
    .get()
    .await
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(started.elapsed() >= Duration::from_secs(1));
}

#[tokio::test]
async fn get_retries_503_with_bounded_backoff() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path("/v1/user"))
        .respond_with(SequenceResponder {
            calls: Arc::clone(&calls),
            first: error_response(503),
            next: ResponseTemplate::new(200).set_body_json(
                json!({ "data": { "user": { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" } } }),
            ),
        })
        .expect(2)
        .mount(&server)
        .await;

    client(
        &server,
        RetryPolicy {
            attempts: 2,
            max_backoff: Duration::ZERO,
        },
    )
    .user()
    .get()
    .await
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn mutation_is_never_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/plans/last-used/payees"))
        .and(body_json(json!({ "payee": { "name": "Market" } })))
        .respond_with(error_response(429).insert_header("Retry-After", "0"))
        .expect(1)
        .mount(&server)
        .await;

    let error = client(
        &server,
        RetryPolicy {
            attempts: 5,
            max_backoff: Duration::ZERO,
        },
    )
    .payees()
    .create(
        &ynab_sdk::PlanId::LastUsed,
        &PostPayee {
            name: "Market".to_owned(),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        ApiError::Response {
            status: StatusCode::TOO_MANY_REQUESTS,
            ..
        }
    ));
}
#[tokio::test]
async fn get_timeout_is_retried_and_classified() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/user"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(50))
                .set_body_json(json!({
                    "data": { "user": { "id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" } }
                })),
        )
        .expect(2)
        .mount(&server)
        .await;
    let error = Client::builder(SecretString::from("fixture-token".to_owned()))
        .base_url(Url::parse(&format!("{}/v1", server.uri())).unwrap())
        .retry_policy(RetryPolicy {
            attempts: 2,
            max_backoff: Duration::ZERO,
        })
        .timeout(Duration::from_millis(10))
        .build()
        .unwrap()
        .user()
        .get()
        .await
        .unwrap_err();
    assert!(matches!(error, ApiError::Timeout));
}

#[tokio::test]
async fn malformed_success_and_error_bodies_are_decode_errors() {
    for (status, body) in [(200, json!({ "user": {} })), (400, json!({ "error": {} }))] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/user"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let error = client(
            &server,
            RetryPolicy {
                attempts: 1,
                max_backoff: Duration::ZERO,
            },
        )
        .user()
        .get()
        .await
        .unwrap_err();
        assert!(matches!(error, ApiError::Decode { .. }));
    }
}

#[tokio::test]
async fn transport_errors_and_debug_output_do_not_expose_credentials() {
    let token = "secret-that-must-not-appear";
    let base_url = Url::parse("http://127.0.0.1:9/private-path").unwrap();
    let client = Client::builder(SecretString::from(token.to_owned()))
        .base_url(base_url)
        .retry_policy(RetryPolicy {
            attempts: 1,
            max_backoff: Duration::ZERO,
        })
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();
    let debug = format!("{client:?}");
    assert!(!debug.contains(token));

    let error = client.user().get().await.unwrap_err();
    let rendered = format!("{error:?} {error}");
    assert!(!rendered.contains(token));
    assert!(!rendered.contains("private-path"));
}

#[test]
fn rate_limit_enforces_api_ceiling() {
    let token = || SecretString::from("fixture-token".to_owned());
    assert!(Client::builder(token()).requests_per_hour(0).is_err());
    assert!(Client::builder(token()).requests_per_hour(200).is_ok());
    assert!(Client::builder(token()).requests_per_hour(201).is_err());
}
