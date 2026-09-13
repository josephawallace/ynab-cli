use reqwest::Method;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    ApiError, Client, PatchPayeeWrapper, PayeePayload, PayeesPayload, PlanId, PostPayee,
    PostPayeeWrapper, SavePayee, SavePayeePayload, ServerKnowledge,
};

#[derive(Clone)]
pub struct PayeesService(Client);

#[derive(Serialize)]
struct LastKnowledge {
    last_knowledge_of_server: ServerKnowledge,
}

impl PayeesService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn list(
        &self,
        plan: &PlanId,
        last_knowledge: Option<ServerKnowledge>,
    ) -> Result<PayeesPayload, ApiError> {
        let plan = plan.to_string();
        match last_knowledge {
            Some(value) => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan, "payees"],
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
                        &["plans", &plan, "payees"],
                        None::<&()>,
                        None::<&()>,
                    )
                    .await
            }
        }
    }

    pub async fn get(&self, plan: &PlanId, payee: Uuid) -> Result<PayeePayload, ApiError> {
        let plan = plan.to_string();
        let payee = payee.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "payees", &payee],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn create(
        &self,
        plan: &PlanId,
        payee: &PostPayee,
    ) -> Result<SavePayeePayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "payees"],
                None::<&()>,
                Some(&PostPayeeWrapper { payee }),
            )
            .await
    }

    pub async fn update(
        &self,
        plan: &PlanId,
        payee: Uuid,
        value: &SavePayee,
    ) -> Result<SavePayeePayload, ApiError> {
        let plan = plan.to_string();
        let payee = payee.to_string();
        self.0
            .execute(
                Method::PATCH,
                &["plans", &plan, "payees", &payee],
                None::<&()>,
                Some(&PatchPayeeWrapper { payee: value }),
            )
            .await
    }
}
