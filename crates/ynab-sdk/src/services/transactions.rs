use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{
    ApiError, Client, CreateTransactions, ExistingTransaction, HybridTransactionsPayload,
    ImportPayload, PatchTransactionsWrapper, PlanId, PlanMonth, PutTransactionWrapper,
    SaveTransactionWithIdOrImportId, SaveTransactionsPayload, TransactionFilters,
    TransactionPayload, TransactionsPayload, ValidationError,
};
#[derive(Clone)]
pub struct TransactionsService(Client);

#[derive(Serialize)]
struct TransactionQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    since_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    kind: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "last_knowledge_of_server"
    )]
    last_knowledge: Option<u64>,
}
impl TransactionsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }
    pub async fn list(
        &self,
        plan: &PlanId,
        filters: &TransactionFilters,
    ) -> Result<TransactionsPayload, ApiError> {
        self.list_at(plan, &["transactions"], filters).await
    }
    pub async fn list_for_account(
        &self,
        plan: &PlanId,
        account: Uuid,
        filters: &TransactionFilters,
    ) -> Result<TransactionsPayload, ApiError> {
        let account = account.to_string();
        self.list_at(plan, &["accounts", &account, "transactions"], filters)
            .await
    }
    pub async fn list_for_category(
        &self,
        plan: &PlanId,
        category: Uuid,
        filters: &TransactionFilters,
    ) -> Result<HybridTransactionsPayload, ApiError> {
        let category = category.to_string();
        self.list_at(plan, &["categories", &category, "transactions"], filters)
            .await
    }
    pub async fn list_for_payee(
        &self,
        plan: &PlanId,
        payee: Uuid,
        filters: &TransactionFilters,
    ) -> Result<HybridTransactionsPayload, ApiError> {
        let payee = payee.to_string();
        self.list_at(plan, &["payees", &payee, "transactions"], filters)
            .await
    }
    pub async fn list_for_month(
        &self,
        plan: &PlanId,
        month: PlanMonth,
        filters: &TransactionFilters,
    ) -> Result<TransactionsPayload, ApiError> {
        let month = month.to_string();
        self.list_at(plan, &["months", &month, "transactions"], filters)
            .await
    }
    async fn list_at<T: DeserializeOwned>(
        &self,
        plan: &PlanId,
        tail: &[&str],
        filters: &TransactionFilters,
    ) -> Result<T, ApiError> {
        let plan = plan.to_string();
        let mut parts = vec!["plans", plan.as_str()];
        parts.extend_from_slice(tail);
        let query = TransactionQuery {
            since_date: filters.since_date.map(|date| date.to_string()),
            until_date: filters.until_date.map(|date| date.to_string()),
            kind: filters.kind.as_ref().map(|kind| kind.0.clone()),
            last_knowledge: filters.last_knowledge.map(|knowledge| knowledge.0),
        };
        self.0
            .execute(Method::GET, &parts, Some(&query), None::<&()>)
            .await
    }
    pub async fn get(
        &self,
        plan: &PlanId,
        transaction: &str,
    ) -> Result<TransactionPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "transactions", transaction],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
    pub async fn create(
        &self,
        plan: &PlanId,
        value: &CreateTransactions,
    ) -> Result<SaveTransactionsPayload, ApiError> {
        value.validate()?;
        let plan = plan.to_string();
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "transactions"],
                None::<&()>,
                Some(value),
            )
            .await
    }
    pub async fn update(
        &self,
        plan: &PlanId,
        transaction: &str,
        value: &ExistingTransaction,
    ) -> Result<TransactionPayload, ApiError> {
        value.validate()?;
        let plan = plan.to_string();
        self.0
            .execute(
                Method::PUT,
                &["plans", &plan, "transactions", transaction],
                None::<&()>,
                Some(&PutTransactionWrapper { transaction: value }),
            )
            .await
    }
    pub async fn update_bulk(
        &self,
        plan: &PlanId,
        values: &[SaveTransactionWithIdOrImportId],
    ) -> Result<SaveTransactionsPayload, ApiError> {
        if values.is_empty() {
            return Err(ApiError::Validation(ValidationError::EmptyTransactions));
        }
        for value in values {
            value.validate()?;
        }
        let plan = plan.to_string();
        self.0
            .execute(
                Method::PATCH,
                &["plans", &plan, "transactions"],
                None::<&()>,
                Some(&PatchTransactionsWrapper {
                    transactions: values,
                }),
            )
            .await
    }
    pub async fn import(&self, plan: &PlanId) -> Result<ImportPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "transactions", "import"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
    pub async fn delete(
        &self,
        plan: &PlanId,
        transaction: &str,
    ) -> Result<TransactionPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::DELETE,
                &["plans", &plan, "transactions", transaction],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
