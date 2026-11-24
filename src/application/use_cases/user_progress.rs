use crate::{
    application::ports::repositories::UserRepository,
    domain::{entities::User, value_objects::Error},
};

const MAX_ATTEMPTS: u32 = 3;

pub async fn update_user_progress<F, R>(
    user_repository: &dyn UserRepository,
    user_id: &str,
    mut apply: F,
) -> Result<R, Error>
where
    F: FnMut(&mut User) -> R,
{
    for _ in 0..MAX_ATTEMPTS {
        let mut user = user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::unauthorized("User not found"))?;

        let expected_version = user.stats.progress_version;
        let result = apply(&mut user);
        user.stats.progress_version = expected_version.wrapping_add(1);

        if user_repository
            .update_progress(&user, expected_version)
            .await?
        {
            return Ok(result);
        }
    }

    Err(Error::conflict("Could not update user stats, try again"))
}
