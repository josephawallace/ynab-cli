use reqwest::Method;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    AccountPayload, AccountsPayload, ApiError, Client, PlanId, PostAccountWrapper, SaveAccount,
    ServerKnowledge,
};

#[derive(Clone)]
pub struct AccountsService(Client);

#[derive(Serialize)]
struct LastKnowledge {
    last_knowledge_of_server: ServerKnowledge,
}

impl AccountsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn list(
        &self,
        plan: &PlanId,
        last_knowledge: Option<ServerKnowledge>,
    ) -> Result<AccountsPayload, ApiError> {
        let plan = plan.to_string();
        match last_knowledge {
            Some(value) => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan, "accounts"],
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
                        &["plans", &plan, "accounts"],
                        None::<&()>,
                        None::<&()>,
                    )
                    .await
            }
        }
    }

    pub async fn get(&self, plan: &PlanId, account: Uuid) -> Result<AccountPayload, ApiError> {
        let plan = plan.to_string();
        let account = account.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "accounts", &account],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn create(
        &self,
        plan: &PlanId,
        account: &SaveAccount,
    ) -> Result<AccountPayload, ApiError> {
        let plan = plan.to_string();
        let body = PostAccountWrapper { account };
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "accounts"],
                None::<&()>,
                Some(&body),
            )
            .await
    }
}
