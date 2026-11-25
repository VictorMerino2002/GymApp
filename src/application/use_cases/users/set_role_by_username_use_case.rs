use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::UserRepository, use_cases::users::change_user_role_use_case::UserRole,
    },
    domain::value_objects::{Error, Role},
};

pub struct SetRoleByUsernameUseCase {
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct SetRoleByUsernameUseCaseParams {
    pub username: String,
    pub role: Role,
}

impl SetRoleByUsernameUseCase {
    pub async fn execute(&self, params: SetRoleByUsernameUseCaseParams) -> Result<UserRole, Error> {
        let user = self
            .user_repository
            .find_by_username(params.username.trim())
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
