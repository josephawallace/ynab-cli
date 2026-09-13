use reqwest::Method;
use uuid::Uuid;

use crate::{ApiError, Client, PayeeLocationPayload, PayeeLocationsPayload, PlanId};

#[derive(Clone)]
pub struct PayeeLocationsService(Client);

impl PayeeLocationsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn list(&self, plan: &PlanId) -> Result<PayeeLocationsPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "payee_locations"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn get(
        &self,
        plan: &PlanId,
        location: Uuid,
    ) -> Result<PayeeLocationPayload, ApiError> {
        let plan = plan.to_string();
        let location = location.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "payee_locations", &location],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn list_for_payee(
        &self,
        plan: &PlanId,
        payee: Uuid,
    ) -> Result<PayeeLocationsPayload, ApiError> {
        let plan = plan.to_string();
        let payee = payee.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "payees", &payee, "payee_locations"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
