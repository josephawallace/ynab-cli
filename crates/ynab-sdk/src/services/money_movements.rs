use crate::{
    ApiError, Client, MoneyMovementGroupsPayload, MoneyMovementsPayload, PlanId, PlanMonth,
};
use reqwest::Method;
#[derive(Clone)]
pub struct MoneyMovementsService(Client);
#[derive(Clone)]
pub struct MoneyMovementGroupsService(Client);
impl MoneyMovementsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }
    pub async fn list(&self, plan: &PlanId) -> Result<MoneyMovementsPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "money_movements"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
    pub async fn list_for_month(
        &self,
        plan: &PlanId,
        month: PlanMonth,
    ) -> Result<MoneyMovementsPayload, ApiError> {
        let plan = plan.to_string();
        let month = month.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "months", &month, "money_movements"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
impl MoneyMovementGroupsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }
    pub async fn list(&self, plan: &PlanId) -> Result<MoneyMovementGroupsPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "money_movement_groups"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
    pub async fn list_for_month(
        &self,
        plan: &PlanId,
        month: PlanMonth,
    ) -> Result<MoneyMovementGroupsPayload, ApiError> {
        let plan = plan.to_string();
        let month = month.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "months", &month, "money_movement_groups"],
                None::<&()>,
                None::<&()>,
            )
            .await
    }
}
