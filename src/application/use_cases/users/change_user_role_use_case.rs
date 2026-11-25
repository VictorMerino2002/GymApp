use std::sync::Arc;

use serde::Serialize;

use crate::{
    application::ports::repositories::UserRepository,
    domain::value_objects::{Error, Role},
};

#[derive(Debug, Serialize)]
pub struct UserRole {
    pub id: String,
    pub username: String,
    pub role: Role,
}

pub struct ChangeUserRoleUseCase {
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct ChangeUserRoleUseCaseParams {
    pub admin_id: String,
    pub user_id: String,
    pub role: Role,
}

impl ChangeUserRoleUseCase {
    pub async fn execute(&self, params: ChangeUserRoleUseCaseParams) -> Result<UserRole, Error> {
        if params.admin_id == params.user_id {
            return Err(Error::forbidden("You cannot change your own role"));
        }

        let user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::not_found("User not found"))?;

        if user.role != params.role {
            self.user_repository
                .update_role(&user.id, params.role)
                .await?;
        }

        Ok(UserRole {
            id: user.id,
            username: user.username,
            role: params.role,
        })
    }
}
