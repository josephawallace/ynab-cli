use reqwest::Method;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    ApiError, Client, PlanId, PostScheduledTransactionWrapper, PutScheduledTransactionWrapper,
    SaveScheduledTransaction, ScheduledTransactionPayload, ScheduledTransactionsPayload,
    ServerKnowledge,
};

#[derive(Clone)]
pub struct ScheduledTransactionsService(Client);

#[derive(Serialize)]
struct LastKnowledge {
    last_knowledge_of_server: ServerKnowledge,
}

impl ScheduledTransactionsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn list(
        &self,
        plan: &PlanId,
        last_knowledge: Option<ServerKnowledge>,
    ) -> Result<ScheduledTransactionsPayload, ApiError> {
        let plan = plan.to_string();
        match last_knowledge {
            Some(value) => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan, "scheduled_transactions"],
                        Some(&LastKnowledge {
                            last_knowledge_of_server: value,
                        }),
                        None::<&()>,
                    )
                    .await
            }
            None => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan, "scheduled_transactions"],
                        None::<&()>,
                        None::<&()>,
                    )
                    .await
            }
        }
    }

    pub async fn get(
        &self,
        plan: &PlanId,
        id: Uuid,
    ) -> Result<ScheduledTransactionPayload, ApiError> {
        let plan = plan.to_string();
        let id = id.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "scheduled_transactions", &id],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn create(
        &self,
        plan: &PlanId,
        value: &SaveScheduledTransaction,
    ) -> Result<ScheduledTransactionPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "scheduled_transactions"],
                None::<&()>,
                Some(&PostScheduledTransactionWrapper {
                    scheduled_transaction: value,
                }),
            )
            .await
    }

    pub async fn update(
        &self,
        plan: &PlanId,
        id: Uuid,
        value: &SaveScheduledTransaction,
    ) -> Result<ScheduledTransactionPayload, ApiError> {
        let plan = plan.to_string();
        let id = id.to_string();
        self.0
            .execute(
                Method::PUT,
                &["plans", &plan, "scheduled_transactions", &id],
                None::<&()>,
                Some(&PutScheduledTransactionWrapper {
                    scheduled_transaction: value,
                }),
            )
            .await
    }

    pub async fn delete(
        &self,
        plan: &PlanId,
        id: Uuid,
    ) -> Result<ScheduledTransactionPayload, ApiError> {
        let plan = plan.to_string();
        let id = id.to_string();
        self.0
            .execute(
                Method::DELETE,
                &["plans", &plan, "scheduled_transactions", &id],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
