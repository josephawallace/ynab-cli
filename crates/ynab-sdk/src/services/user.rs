use reqwest::Method;

use crate::{ApiError, Client, UserPayload};

#[derive(Clone)]
pub struct UserService(Client);

impl UserService {
    pub(crate) fn new(client: Client) -> Self {
        Self(client)
    }

    pub async fn get(&self) -> Result<UserPayload, ApiError> {
        self.0
            .execute::<UserPayload, (), ()>(Method::GET, &["user"], None, None)
            .await
    }
}
