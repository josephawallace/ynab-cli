use time::macros::date;

use secrecy::SecretString;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};
use ynab_sdk::{Client, PlanId, ServerKnowledge, TransactionFilters, TransactionType};

#[tokio::test]
async fn category_transaction_list_uses_only_category_route_and_filters() {
    let server = MockServer::start().await;
    let plan = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000011").expect("fixture UUID");
    let category =
        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000012").expect("fixture UUID");
    Mock::given(method("GET"))
        .and(path(format!(
            "/plans/{plan}/categories/{category}/transactions"
        )))
        .and(query_param("since_date", "2026-01-01"))
        .and(query_param("until_date", "2026-01-31"))
        .and(query_param("type", "uncategorized"))
        .and(query_param("last_knowledge_of_server", "7"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data":{"transactions":[],"server_knowledge":8}}),
            ),
        )
        .mount(&server)
        .await;
    let client = Client::builder(SecretString::from("synthetic-test-secret"))
        .base_url(Url::parse(&server.uri()).expect("wiremock URL"))
        .build()
        .expect("client");
    let filters = TransactionFilters {
        since_date: Some(date!(2026 - 01 - 01)),
        until_date: Some(date!(2026 - 01 - 31)),
        kind: Some(TransactionType("uncategorized".into())),
        last_knowledge: Some(ServerKnowledge(7)),
    };
    let result = client
        .transactions()
        .list_for_category(&PlanId::Id(plan), category, &filters)
        .await
        .expect("successful response");
    assert!(result.transactions.is_empty());
    assert_eq!(result.server_knowledge, Some(ServerKnowledge(8)));
}
