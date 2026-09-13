use secrecy::SecretString;
use serde_json::json;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};
use ynab_sdk::{Client, PlanId, PostPayee};

#[tokio::test]
async fn create_payee_wraps_body_and_decodes_payload() {
    let server = MockServer::start().await;
    let plan = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("fixture UUID");
    let payee =
        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("fixture UUID");
    Mock::given(method("POST"))
        .and(path(format!("/plans/{plan}/payees")))
        .and(body_json(json!({"payee":{"name":"Acme"}})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "data": {
                "payee": {"id":payee,"name":"Acme","transfer_account_id":null,"deleted":false},
                "server_knowledge": 1
            }
        })))
        .mount(&server)
        .await;
    let client = Client::builder(SecretString::from("synthetic-test-secret"))
        .base_url(Url::parse(&server.uri()).expect("wiremock URL"))
        .build()
        .expect("client");
    let result = client
        .payees()
        .create(
            &PlanId::Id(plan),
            &PostPayee {
                name: "Acme".into(),
            },
        )
        .await
        .expect("successful response");
    assert_eq!(result.payee.id, payee);
    assert_eq!(result.payee.name, "Acme");
}
