use reqwest::Method;
use serde::Serialize;

use crate::{
    ApiError, Client, PlanDetailPayload, PlanId, PlanSettingsPayload, PlansPayload, ServerKnowledge,
};

#[derive(Clone)]
pub struct PlansService(Client);

#[derive(Serialize)]
struct IncludeAccounts {
    include_accounts: bool,
}

#[derive(Serialize)]
struct LastKnowledge {
    last_knowledge_of_server: ServerKnowledge,
}

impl PlansService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn list(&self, include_accounts: bool) -> Result<PlansPayload, ApiError> {
        self.0
            .execute(
                Method::GET,
                &["plans"],
                Some(&IncludeAccounts { include_accounts }),
                None::<&()>,
            )
            .await
    }

    pub async fn get(
        &self,
        plan: &PlanId,
        last_knowledge: Option<ServerKnowledge>,
    ) -> Result<PlanDetailPayload, ApiError> {
        let plan = plan.to_string();
        match last_knowledge {
            Some(value) => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan],
                        Some(&LastKnowledge {
                            last_knowledge_of_server: value,
                        }),
                        None::<&()>,
                    )
                    .await
            }
            None => {
                self.0
                    .execute(Method::GET, &["plans", &plan], None::<&()>, None::<&()>)
                    .await
            }
        }
    }

    pub async fn settings(&self, plan: &PlanId) -> Result<PlanSettingsPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "settings"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
