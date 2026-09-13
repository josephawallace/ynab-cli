use reqwest::Method;
use uuid::Uuid;

use crate::{
    ApiError, Client, PatchCategoryGroupWrapper, PlanId, PostCategoryGroupWrapper,
    SaveCategoryGroup, SaveCategoryGroupPayload,
};

#[derive(Clone)]
pub struct CategoryGroupsService(Client);

impl CategoryGroupsService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn create(
        &self,
        plan: &PlanId,
        group: &SaveCategoryGroup,
    ) -> Result<SaveCategoryGroupPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "category_groups"],
                None::<&()>,
                Some(&PostCategoryGroupWrapper {
                    category_group: group,
                }),
            )
            .await
    }

    pub async fn update(
        &self,
        plan: &PlanId,
        group: Uuid,
        value: &SaveCategoryGroup,
    ) -> Result<SaveCategoryGroupPayload, ApiError> {
        let plan = plan.to_string();
        let group = group.to_string();
        self.0
            .execute(
                Method::PATCH,
                &["plans", &plan, "category_groups", &group],
                None::<&()>,
                Some(&PatchCategoryGroupWrapper {
                    category_group: value,
                }),
            )
            .await
    }
}
