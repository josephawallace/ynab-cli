use reqwest::Method;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    ApiError, CategoriesPayload, CategoryPayload, Client, NewCategory, PatchCategoryWrapper,
    PatchMonthCategoryWrapper, PlanId, PlanMonth, PostCategoryWrapper, SaveCategory,
    SaveCategoryPayload, SaveMonthCategory, ServerKnowledge,
};

#[derive(Clone)]
pub struct CategoriesService(Client);

#[derive(Serialize)]
struct LastKnowledge {
    last_knowledge_of_server: ServerKnowledge,
}

impl CategoriesService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn list(
        &self,
        plan: &PlanId,
        last_knowledge: Option<ServerKnowledge>,
    ) -> Result<CategoriesPayload, ApiError> {
        let plan = plan.to_string();
        match last_knowledge {
            Some(value) => {
                self.0
                    .execute(
                        Method::GET,
                        &["plans", &plan, "categories"],
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
                        &["plans", &plan, "categories"],
                        None::<&()>,
                        None::<&()>,
                    )
                    .await
            }
        }
    }

    pub async fn get(&self, plan: &PlanId, category: Uuid) -> Result<CategoryPayload, ApiError> {
        let plan = plan.to_string();
        let category = category.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "categories", &category],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn create(
        &self,
        plan: &PlanId,
        category: &NewCategory,
    ) -> Result<SaveCategoryPayload, ApiError> {
        let plan = plan.to_string();
        self.0
            .execute(
                Method::POST,
                &["plans", &plan, "categories"],
                None::<&()>,
                Some(&PostCategoryWrapper { category }),
            )
            .await
    }

    pub async fn update(
        &self,
        plan: &PlanId,
        category: Uuid,
        value: &SaveCategory,
    ) -> Result<SaveCategoryPayload, ApiError> {
        let plan = plan.to_string();
        let category = category.to_string();
        self.0
            .execute(
                Method::PATCH,
                &["plans", &plan, "categories", &category],
                None::<&()>,
                Some(&PatchCategoryWrapper { category: value }),
            )
            .await
    }

    pub async fn get_month(
        &self,
        plan: &PlanId,
        month: PlanMonth,
        category: Uuid,
    ) -> Result<CategoryPayload, ApiError> {
        let plan = plan.to_string();
        let month = month.to_string();
        let category = category.to_string();
        self.0
            .execute(
                Method::GET,
                &["plans", &plan, "months", &month, "categories", &category],
                None::<&()>,
                None::<&()>,
            )
            .await
    }

    pub async fn update_month(
        &self,
        plan: &PlanId,
        month: PlanMonth,
        category: Uuid,
        value: &SaveMonthCategory,
    ) -> Result<SaveCategoryPayload, ApiError> {
        let plan = plan.to_string();
        let month = month.to_string();
        let category = category.to_string();
        self.0
            .execute(
                Method::PATCH,
                &["plans", &plan, "months", &month, "categories", &category],
                None::<&()>,
                Some(&PatchMonthCategoryWrapper { category: value }),
            )
            .await
    }
}
