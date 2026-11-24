use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;

use crate::{
    application::{
        ports::repositories::{SessionRepository, UserRepository},
        use_cases::user_progress::update_user_progress,
    },
    domain::value_objects::Error,
};

#[derive(Debug, Serialize)]
pub struct SessionCompletion {
    pub session_id: String,
    pub xp_awarded: u32,
    pub xp: u32,
    pub level: u32,
    pub sessions_done: u32,
}

pub struct CompleteSessionUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct CompleteSessionUseCaseParams {
    pub user_id: String,
    pub session_id: String,
}

impl CompleteSessionUseCase {
    pub async fn execute(
        &self,
        params: CompleteSessionUseCaseParams,
    ) -> Result<SessionCompletion, Error> {
        self.session_repository
            .find_by_id(&params.session_id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))?;

        let today = Utc::now().date_naive();
        let (xp_awarded, stats) =
            update_user_progress(&*self.user_repository, &params.user_id, |user| {
                let xp_awarded = user.complete_session(today);
                (xp_awarded, user.stats)
            })
            .await?;

        Ok(SessionCompletion {
            session_id: params.session_id,
            xp_awarded,
            xp: stats.xp,
            level: stats.level(),
            sessions_done: stats.sessions_done,
        })
    }
}
