use std::sync::Arc;

use crate::{
    application::ports::{
        PasswordHasherPort,
        repositories::{
            ExerciseHistoryRepository, FavRoutineRepository, FavSessionRepository,
            FriendshipRepository, UserRepository,
        },
    },
    domain::value_objects::Error,
};

pub struct DeleteAccountUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub friendship_repository: Arc<dyn FriendshipRepository>,
    pub fav_routine_repository: Arc<dyn FavRoutineRepository>,
    pub fav_session_repository: Arc<dyn FavSessionRepository>,
    pub exercise_history_repository: Arc<dyn ExerciseHistoryRepository>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
}

pub struct DeleteAccountUseCaseParams {
    pub user_id: String,
    pub password: String,
}

impl DeleteAccountUseCase {
    pub async fn execute(&self, params: DeleteAccountUseCaseParams) -> Result<(), Error> {
        let user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::unauthorized("User not found"))?;

        let password_hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| Error::unauthorized("Invalid credentials"))?;

        if !self
            .password_hasher
            .verify(&params.password, password_hash)
            .await?
        {
            return Err(Error::unauthorized("Invalid credentials"));
        }

        tokio::try_join!(
            self.friendship_repository.delete_by_user(&user.id),
            self.fav_routine_repository.delete_by_user(&user.id),
            self.fav_session_repository.delete_by_user(&user.id),
            self.exercise_history_repository.delete_by_user(&user.id),
        )?;

        self.user_repository.delete(&user).await
    }
}
