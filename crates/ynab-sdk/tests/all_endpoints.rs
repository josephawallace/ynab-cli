use secrecy::SecretString;
use serde_json::{Value, json};
use time::macros::date;
use url::Url;
use uuid::Uuid;
use wiremock::{
    Match, Mock, MockServer, Request, ResponseTemplate,
    matchers::{body_bytes, body_json, header, method, path},
};
use ynab_sdk::{
    Client, CreateTransactions, ExistingTransaction, Milliunits, NewCategory, NewTransaction,
    Nullable, PlanId, PlanMonth, PostPayee, SaveAccount, SaveAccountType, SaveCategory,
    SaveCategoryGroup, SaveMonthCategory, SavePayee, SaveScheduledTransaction,
    SaveTransactionWithIdOrImportId, ScheduledTransactionFrequency, ServerKnowledge,
    TransactionFilters, TransactionType,
};

const PLAN: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const ACCOUNT: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const CATEGORY: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";
const GROUP: &str = "dddddddd-dddd-dddd-dddd-dddddddddddd";
const PAYEE: &str = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
const LOCATION: &str = "ffffffff-ffff-ffff-ffff-ffffffffffff";
const TRANSACTION: &str = "11111111-1111-1111-1111-111111111111";
const SCHEDULED: &str = "22222222-2222-2222-2222-222222222222";
const MOVEMENT: &str = "33333333-3333-3333-3333-333333333333";
const MOVEMENT_GROUP: &str = "44444444-4444-4444-4444-444444444444";

#[derive(Clone)]
struct ExactQuery(Vec<(String, String)>);

impl Match for ExactQuery {
    fn matches(&self, request: &Request) -> bool {
        let mut actual: Vec<_> = request
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        let mut expected = self.0.clone();
        actual.sort_unstable();
        expected.sort_unstable();
        actual == expected
    }
}

fn client(server: &MockServer) -> Client {
    Client::builder(SecretString::from("fixture-token".to_owned()))
        .base_url(Url::parse(&format!("{}/v1", server.uri())).unwrap())
        .build()
        .unwrap()
}

async fn mount(
    server: &MockServer,
    verb: &str,
    route: &str,
    query: &[(&str, &str)],
    body: Option<Value>,
    data: Value,
) {
    let mock = Mock::given(method(verb))
        .and(path(route))
        .and(header("authorization", "Bearer fixture-token"))
        .and(ExactQuery(
            query
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        ));
    let mock = if let Some(body) = body {
        mock.and(body_json(body))
    } else {
        mock.and(body_bytes(Vec::<u8>::new()))
    };
    mock.respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": data })))
        .expect(1)
        .mount(server)
        .await;
}

fn id(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}

fn plan_id() -> PlanId {
    PlanId::Id(id(PLAN))
}

fn month() -> PlanMonth {
    PlanMonth::new(date!(2026 - 01 - 01)).unwrap()
}

fn account_json() -> Value {
    json!({
        "id": ACCOUNT, "name": "Checking", "type": "checking", "on_budget": true,
        "closed": false, "note": null, "balance": 0, "cleared_balance": 0,
        "uncleared_balance": 0, "transfer_payee_id": null, "deleted": false
    })
}

fn category_json() -> Value {
    json!({
        "id": CATEGORY, "category_group_id": GROUP, "name": "Food", "hidden": false,
        "internal": false, "note": null, "budgeted": 0, "activity": 0,
        "balance": 0, "deleted": false
    })
}

fn group_json() -> Value {
    json!({ "id": GROUP, "name": "Needs", "hidden": false, "internal": false, "deleted": false })
}

fn payee_json() -> Value {
    json!({ "id": PAYEE, "name": "Market", "transfer_account_id": null, "deleted": false })
}

fn location_json() -> Value {
    json!({
        "id": LOCATION, "payee_id": PAYEE, "latitude": "51.5", "longitude": "-0.1",
        "deleted": false
    })
}

fn month_json() -> Value {
    json!({
        "month": "2026-01-01", "note": null, "income": 0, "budgeted": 0,
        "activity": 0, "to_be_budgeted": 0, "age_of_money": null, "deleted": false
    })
}

fn transaction_json() -> Value {
    json!({
        "id": TRANSACTION, "date": "2026-01-02", "amount": -1250, "memo": null,
        "cleared": "uncleared", "approved": false, "flag_color": null,
        "flag_name": null, "account_id": ACCOUNT, "payee_id": null,
        "category_id": null, "transfer_account_id": null, "transfer_transaction_id": null,
        "matched_transaction_id": null, "import_id": null, "import_payee_name": null,
        "import_payee_name_original": null, "debt_transaction_type": null, "deleted": false,
        "account_name": "Checking", "payee_name": null, "category_name": null,
        "subtransactions": []
    })
}

fn hybrid_transaction_json() -> Value {
    let mut value = transaction_json();
    let object = value.as_object_mut().unwrap();
    object.insert("type".to_owned(), json!("transaction"));
    object.insert("parent_transaction_id".to_owned(), Value::Null);
    object.insert("category_name".to_owned(), json!("Food"));
    object.remove("subtransactions");
    value
}

fn scheduled_json() -> Value {
    json!({
        "id": SCHEDULED, "date_first": "2026-01-02", "date_next": "2026-02-02",
        "frequency": "monthly", "amount": -1250, "memo": null, "flag_color": null,
        "account_id": ACCOUNT, "payee_id": null, "category_id": null,
        "transfer_account_id": null, "deleted": false, "account_name": "Checking",
        "payee_name": null, "category_name": null, "subtransactions": []
    })
}

#[tokio::test]
async fn user_and_plan_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    mount(
        &server,
        "GET",
        "/v1/user",
        &[],
        None,
        json!({ "user": { "id": PAYEE } }),
    )
    .await;
    assert_eq!(client.user().get().await.unwrap().user.id, id(PAYEE));

    mount(
        &server,
        "GET",
        "/v1/plans",
        &[("include_accounts", "true")],
        None,
        json!({ "plans": [{ "id": PLAN, "name": "Budget" }], "default_plan": { "id": PLAN, "name": "Budget" } }),
    )
    .await;
    let plans = client.plans().list(true).await.unwrap();
    assert_eq!(plans.default_plan.unwrap().id, id(PLAN));

    mount(
        &server,
        "GET",
        &format!("/v1/plans/{PLAN}"),
        &[("last_knowledge_of_server", "7")],
        None,
        json!({ "plan": { "id": PLAN, "name": "Budget" }, "server_knowledge": 8 }),
    )
    .await;
    assert_eq!(
        client
            .plans()
            .get(&plan_id(), Some(ServerKnowledge(7)))
            .await
            .unwrap()
            .server_knowledge,
        ServerKnowledge(8)
    );

    mount(
        &server,
        "GET",
        &format!("/v1/plans/{PLAN}/settings"),
        &[],
        None,
        json!({
            "settings": {
                "date_format": { "format": "DD/MM/YYYY" },
                "currency_format": {
                    "iso_code": "USD", "example_format": "123.45", "decimal_digits": 2,
                    "decimal_separator": ".", "symbol_first": true, "group_separator": ",",
                    "currency_symbol": "$", "display_symbol": true
                }
            }
        }),
    )
    .await;
    assert_eq!(
        client
            .plans()
            .settings(&plan_id())
            .await
            .unwrap()
            .settings
            .currency_format
            .iso_code,
        "USD"
    );
}

#[tokio::test]
async fn account_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    let list_route = format!("/v1/plans/{PLAN}/accounts");
    mount(
        &server,
        "GET",
        &list_route,
        &[("last_knowledge_of_server", "3")],
        None,
        json!({ "accounts": [account_json()], "server_knowledge": 4 }),
    )
    .await;
    assert_eq!(
        client
            .accounts()
            .list(&plan_id(), Some(ServerKnowledge(3)))
            .await
            .unwrap()
            .accounts
            .len(),
        1
    );

    mount(
        &server,
        "GET",
        &format!("{list_route}/{ACCOUNT}"),
        &[],
        None,
        json!({ "account": account_json() }),
    )
    .await;
    assert_eq!(
        client
            .accounts()
            .get(&plan_id(), id(ACCOUNT))
            .await
            .unwrap()
            .account
            .base
            .id,
        id(ACCOUNT)
    );

    let request = SaveAccount {
        name: "Checking".to_owned(),
        account_type: SaveAccountType::CHECKING.to_owned().into(),
        balance: Milliunits(0),
    };
    mount(
        &server,
        "POST",
        &list_route,
        &[],
        Some(json!({ "account": { "name": "Checking", "type": "checking", "balance": 0 } })),
        json!({ "account": account_json() }),
    )
    .await;
    client
        .accounts()
        .create(&plan_id(), &request)
        .await
        .unwrap();
}

#[tokio::test]
async fn category_and_group_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    let route = format!("/v1/plans/{PLAN}/categories");
    mount(
        &server,
        "GET",
        &route,
        &[("last_knowledge_of_server", "4")],
        None,
        json!({
            "category_groups": [{
                "id": GROUP, "name": "Needs", "hidden": false, "internal": false,
                "deleted": false, "categories": [category_json()]
            }],
            "server_knowledge": 5
        }),
    )
    .await;
    client
        .categories()
        .list(&plan_id(), Some(ServerKnowledge(4)))
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("{route}/{CATEGORY}"),
        &[],
        None,
        json!({ "category": category_json() }),
    )
    .await;
    client
        .categories()
        .get(&plan_id(), id(CATEGORY))
        .await
        .unwrap();

    let new_category = NewCategory {
        name: Some("Food".to_owned()),
        note: Nullable::Null,
        category_group_id: id(GROUP),
        goal_target: Nullable::Omitted,
        goal_target_date: Nullable::Omitted,
        goal_needs_whole_amount: Nullable::Omitted,
        goal_frequency: None,
    };
    mount(
        &server,
        "POST",
        &route,
        &[],
        Some(json!({ "category": { "name": "Food", "note": null, "category_group_id": GROUP } })),
        json!({ "category": category_json(), "server_knowledge": 6 }),
    )
    .await;
    client
        .categories()
        .create(&plan_id(), &new_category)
        .await
        .unwrap();

    let save_category = SaveCategory {
        name: Nullable::Value("Groceries".to_owned()),
        ..SaveCategory::default()
    };
    mount(
        &server,
        "PATCH",
        &format!("{route}/{CATEGORY}"),
        &[],
        Some(json!({ "category": { "name": "Groceries" } })),
        json!({ "category": category_json(), "server_knowledge": 7 }),
    )
    .await;
    client
        .categories()
        .update(&plan_id(), id(CATEGORY), &save_category)
        .await
        .unwrap();

    let month_route = format!("/v1/plans/{PLAN}/months/2026-01-01/categories/{CATEGORY}");
    mount(
        &server,
        "GET",
        &month_route,
        &[],
        None,
        json!({ "category": category_json() }),
    )
    .await;
    client
        .categories()
        .get_month(&plan_id(), month(), id(CATEGORY))
        .await
        .unwrap();

    mount(
        &server,
        "PATCH",
        &month_route,
        &[],
        Some(json!({ "category": { "budgeted": 25000 } })),
        json!({ "category": category_json(), "server_knowledge": 8 }),
    )
    .await;
    client
        .categories()
        .update_month(
            &plan_id(),
            month(),
            id(CATEGORY),
            &SaveMonthCategory {
                budgeted: Milliunits(25_000),
            },
        )
        .await
        .unwrap();

    let groups = format!("/v1/plans/{PLAN}/category_groups");
    let group_request = SaveCategoryGroup {
        name: "Wants".to_owned(),
    };
    mount(
        &server,
        "POST",
        &groups,
        &[],
        Some(json!({ "category_group": { "name": "Wants" } })),
        json!({ "category_group": group_json(), "server_knowledge": 9 }),
    )
    .await;
    client
        .category_groups()
        .create(&plan_id(), &group_request)
        .await
        .unwrap();

    mount(
        &server,
        "PATCH",
        &format!("{groups}/{GROUP}"),
        &[],
        Some(json!({ "category_group": { "name": "Wants" } })),
        json!({ "category_group": group_json(), "server_knowledge": 10 }),
    )
    .await;
    client
        .category_groups()
        .update(&plan_id(), id(GROUP), &group_request)
        .await
        .unwrap();
}

#[tokio::test]
async fn payee_and_location_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    let route = format!("/v1/plans/{PLAN}/payees");
    mount(
        &server,
        "GET",
        &route,
        &[("last_knowledge_of_server", "10")],
        None,
        json!({ "payees": [payee_json()], "server_knowledge": 11 }),
    )
    .await;
    client
        .payees()
        .list(&plan_id(), Some(ServerKnowledge(10)))
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("{route}/{PAYEE}"),
        &[],
        None,
        json!({ "payee": payee_json() }),
    )
    .await;
    client.payees().get(&plan_id(), id(PAYEE)).await.unwrap();

    mount(
        &server,
        "POST",
        &route,
        &[],
        Some(json!({ "payee": { "name": "Market" } })),
        json!({ "payee": payee_json(), "server_knowledge": 12 }),
    )
    .await;
    client
        .payees()
        .create(
            &plan_id(),
            &PostPayee {
                name: "Market".to_owned(),
            },
        )
        .await
        .unwrap();

    mount(
        &server,
        "PATCH",
        &format!("{route}/{PAYEE}"),
        &[],
        Some(json!({ "payee": { "name": "Shop" } })),
        json!({ "payee": payee_json(), "server_knowledge": 13 }),
    )
    .await;
    client
        .payees()
        .update(
            &plan_id(),
            id(PAYEE),
            &SavePayee {
                name: Some("Shop".to_owned()),
            },
        )
        .await
        .unwrap();

    let locations = format!("/v1/plans/{PLAN}/payee_locations");
    mount(
        &server,
        "GET",
        &locations,
        &[],
        None,
        json!({ "payee_locations": [location_json()] }),
    )
    .await;
    client.payee_locations().list(&plan_id()).await.unwrap();

    mount(
        &server,
        "GET",
        &format!("{locations}/{LOCATION}"),
        &[],
        None,
        json!({ "payee_location": location_json() }),
    )
    .await;
    client
        .payee_locations()
        .get(&plan_id(), id(LOCATION))
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("{route}/{PAYEE}/payee_locations"),
        &[],
        None,
        json!({ "payee_locations": [location_json()] }),
    )
    .await;
    client
        .payee_locations()
        .list_for_payee(&plan_id(), id(PAYEE))
        .await
        .unwrap();
}

#[tokio::test]
async fn month_and_money_movement_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    let months = format!("/v1/plans/{PLAN}/months");
    mount(
        &server,
        "GET",
        &months,
        &[("last_knowledge_of_server", "14")],
        None,
        json!({ "months": [month_json()], "server_knowledge": 15 }),
    )
    .await;
    client
        .months()
        .list(&plan_id(), Some(ServerKnowledge(14)))
        .await
        .unwrap();

    let mut detail = month_json();
    detail
        .as_object_mut()
        .unwrap()
        .insert("categories".to_owned(), json!([category_json()]));
    mount(
        &server,
        "GET",
        &format!("{months}/2026-01-01"),
        &[],
        None,
        json!({ "month": detail }),
    )
    .await;
    client.months().get(&plan_id(), month()).await.unwrap();

    let movement = json!({ "id": MOVEMENT, "amount": 1000 });
    let movements = format!("/v1/plans/{PLAN}/money_movements");
    mount(
        &server,
        "GET",
        &movements,
        &[],
        None,
        json!({ "money_movements": [movement.clone()], "server_knowledge": 16 }),
    )
    .await;
    client.money_movements().list(&plan_id()).await.unwrap();

    mount(
        &server,
        "GET",
        &format!("{months}/2026-01-01/money_movements"),
        &[],
        None,
        json!({ "money_movements": [movement], "server_knowledge": 17 }),
    )
    .await;
    client
        .money_movements()
        .list_for_month(&plan_id(), month())
        .await
        .unwrap();

    let movement_group = json!({
        "id": MOVEMENT_GROUP, "group_created_at": "2026-01-02T03:04:05Z",
        "month": "2026-01-01"
    });
    let groups = format!("/v1/plans/{PLAN}/money_movement_groups");
    mount(
        &server,
        "GET",
        &groups,
        &[],
        None,
        json!({ "money_movement_groups": [movement_group.clone()], "server_knowledge": 18 }),
    )
    .await;
    client
        .money_movement_groups()
        .list(&plan_id())
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("{months}/2026-01-01/money_movement_groups"),
        &[],
        None,
        json!({ "money_movement_groups": [movement_group], "server_knowledge": 19 }),
    )
    .await;
    client
        .money_movement_groups()
        .list_for_month(&plan_id(), month())
        .await
        .unwrap();
}

#[tokio::test]
async fn transaction_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    let filters = TransactionFilters {
        since_date: Some(date!(2026 - 01 - 01)),
        until_date: Some(date!(2026 - 01 - 31)),
        kind: Some(TransactionType::UNCATEGORIZED.to_owned().into()),
        last_knowledge: Some(ServerKnowledge(19)),
    };
    let query = [
        ("since_date", "2026-01-01"),
        ("until_date", "2026-01-31"),
        ("type", "uncategorized"),
        ("last_knowledge_of_server", "19"),
    ];
    let transactions = format!("/v1/plans/{PLAN}/transactions");
    mount(
        &server,
        "GET",
        &transactions,
        &query,
        None,
        json!({ "transactions": [transaction_json()], "server_knowledge": 20 }),
    )
    .await;
    client
        .transactions()
        .list(&plan_id(), &filters)
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("/v1/plans/{PLAN}/accounts/{ACCOUNT}/transactions"),
        &query,
        None,
        json!({ "transactions": [transaction_json()], "server_knowledge": 21 }),
    )
    .await;
    client
        .transactions()
        .list_for_account(&plan_id(), id(ACCOUNT), &filters)
        .await
        .unwrap();

    for (route, entity) in [
        (
            format!("/v1/plans/{PLAN}/categories/{CATEGORY}/transactions"),
            id(CATEGORY),
        ),
        (
            format!("/v1/plans/{PLAN}/payees/{PAYEE}/transactions"),
            id(PAYEE),
        ),
    ] {
        mount(
            &server,
            "GET",
            &route,
            &query,
            None,
            json!({ "transactions": [hybrid_transaction_json()], "server_knowledge": 22 }),
        )
        .await;
        if route.contains("categories") {
            client
                .transactions()
                .list_for_category(&plan_id(), entity, &filters)
                .await
                .unwrap();
        } else {
            client
                .transactions()
                .list_for_payee(&plan_id(), entity, &filters)
                .await
                .unwrap();
        }
    }

    mount(
        &server,
        "GET",
        &format!("/v1/plans/{PLAN}/months/2026-01-01/transactions"),
        &query,
        None,
        json!({ "transactions": [transaction_json()], "server_knowledge": 23 }),
    )
    .await;
    client
        .transactions()
        .list_for_month(&plan_id(), month(), &filters)
        .await
        .unwrap();

    let one = ExistingTransaction {
        account_id: Some(id(ACCOUNT)),
        date: Some(date!(2026 - 01 - 02)),
        amount: Some(Milliunits(-1250)),
        memo: Nullable::Null,
        ..ExistingTransaction::default()
    };
    mount(
        &server,
        "POST",
        &transactions,
        &[],
        Some(json!({
            "transaction": {
                "account_id": ACCOUNT, "date": "2026-01-02", "amount": -1250, "memo": null
            }
        })),
        json!({ "transaction_ids": [TRANSACTION], "transaction": transaction_json(), "server_knowledge": 24 }),
    )
    .await;
    client
        .transactions()
        .create(
            &plan_id(),
            &CreateTransactions::One(NewTransaction {
                transaction: one.clone(),
                import_id: Nullable::Omitted,
            }),
        )
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("{transactions}/{TRANSACTION}"),
        &[],
        None,
        json!({ "transaction": transaction_json(), "server_knowledge": 25 }),
    )
    .await;
    client
        .transactions()
        .get(&plan_id(), id(TRANSACTION))
        .await
        .unwrap();

    let update = ExistingTransaction {
        memo: Nullable::Value("updated".to_owned()),
        ..ExistingTransaction::default()
    };
    mount(
        &server,
        "PUT",
        &format!("{transactions}/{TRANSACTION}"),
        &[],
        Some(json!({ "transaction": { "memo": "updated" } })),
        json!({ "transaction": transaction_json(), "server_knowledge": 26 }),
    )
    .await;
    client
        .transactions()
        .update(&plan_id(), id(TRANSACTION), &update)
        .await
        .unwrap();

    mount(
        &server,
        "PATCH",
        &transactions,
        &[],
        Some(json!({ "transactions": [{ "id": TRANSACTION, "memo": "updated" }] })),
        json!({ "transaction_ids": [TRANSACTION], "transactions": [transaction_json()], "server_knowledge": 27 }),
    )
    .await;
    client
        .transactions()
        .update_bulk(
            &plan_id(),
            &[SaveTransactionWithIdOrImportId {
                id: Some(id(TRANSACTION)),
                import_id: None,
                transaction: update,
            }],
        )
        .await
        .unwrap();

    mount(
        &server,
        "POST",
        &format!("{transactions}/import"),
        &[],
        None,
        json!({ "transaction_ids": [TRANSACTION] }),
    )
    .await;
    client.transactions().import(&plan_id()).await.unwrap();

    mount(
        &server,
        "DELETE",
        &format!("{transactions}/{TRANSACTION}"),
        &[],
        None,
        json!({ "transaction": transaction_json(), "server_knowledge": 28 }),
    )
    .await;
    client
        .transactions()
        .delete(&plan_id(), id(TRANSACTION))
        .await
        .unwrap();
}

#[tokio::test]
async fn scheduled_transaction_endpoints_match_contract() {
    let server = MockServer::start().await;
    let client = client(&server);
    let route = format!("/v1/plans/{PLAN}/scheduled_transactions");
    mount(
        &server,
        "GET",
        &route,
        &[("last_knowledge_of_server", "28")],
        None,
        json!({ "scheduled_transactions": [scheduled_json()], "server_knowledge": 29 }),
    )
    .await;
    client
        .scheduled_transactions()
        .list(&plan_id(), Some(ServerKnowledge(28)))
        .await
        .unwrap();

    mount(
        &server,
        "GET",
        &format!("{route}/{SCHEDULED}"),
        &[],
        None,
        json!({ "scheduled_transaction": scheduled_json() }),
    )
    .await;
    client
        .scheduled_transactions()
        .get(&plan_id(), id(SCHEDULED))
        .await
        .unwrap();

    let request = SaveScheduledTransaction {
        account_id: id(ACCOUNT),
        date: date!(2026 - 01 - 02),
        amount: Some(Milliunits(-1250)),
        payee_id: Nullable::Omitted,
        payee_name: Nullable::Omitted,
        category_id: Nullable::Omitted,
        memo: Nullable::Null,
        flag_color: Nullable::Omitted,
        frequency: Some(ScheduledTransactionFrequency::MONTHLY.to_owned().into()),
    };
    let body = json!({
        "scheduled_transaction": {
            "account_id": ACCOUNT, "date": "2026-01-02", "amount": -1250,
            "memo": null, "frequency": "monthly"
        }
    });
    mount(
        &server,
        "POST",
        &route,
        &[],
        Some(body.clone()),
        json!({ "scheduled_transaction": scheduled_json() }),
    )
    .await;
    client
        .scheduled_transactions()
        .create(&plan_id(), &request)
        .await
        .unwrap();

    mount(
        &server,
        "PUT",
        &format!("{route}/{SCHEDULED}"),
        &[],
        Some(body),
        json!({ "scheduled_transaction": scheduled_json() }),
    )
    .await;
    client
        .scheduled_transactions()
        .update(&plan_id(), id(SCHEDULED), &request)
        .await
        .unwrap();

    mount(
        &server,
        "DELETE",
        &format!("{route}/{SCHEDULED}"),
        &[],
        None,
        json!({ "scheduled_transaction": scheduled_json() }),
    )
    .await;
    client
        .scheduled_transactions()
        .delete(&plan_id(), id(SCHEDULED))
        .await
        .unwrap();
}
