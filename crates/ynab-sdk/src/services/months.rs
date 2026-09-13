use crate::{ApiError, Client, MonthPayload, MonthsPayload, PlanId, PlanMonth, ServerKnowledge};
use reqwest::Method;
use serde::Serialize;
#[derive(Clone)]
pub struct MonthsService(Client);
#[derive(Serialize)]
struct LastKnowledge {
    last_knowledge_of_server: ServerKnowledge,
}
impl MonthsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }
    pub async fn list(
        &self,
        plan: &PlanId,
        last_knowledge: Option<ServerKnowledge>,
    ) -> Result<MonthsPayload, ApiError> {
        let plan = plan.to_string();
        match last_knowledge {
            Some(value) => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan, "months"],
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
                        &["plans", &plan, "months"],
                        None::<&()>,
                        None::<&()>,
                    )
                    .await
            }
        }
    }
    pub async fn get(&self, plan: &PlanId, month: PlanMonth) -> Result<MonthPayload, ApiError> {
        let plan = plan.to_string();
        let month = month.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "months", &month],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
