use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;

use crate::{
    application::{
        ports::repositories::{ActivityRepository, SessionRepository, UserRepository},
        use_cases::{
            activity_publisher::{level_up, publish_activities},
            user_progress::update_user_progress,
        },
    },
    domain::{entities::ActivityKind, value_objects::Error},
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
    pub activity_repository: Arc<dyn ActivityRepository>,
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
        let session = self
            .session_repository
            .find_by_id(&params.session_id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))?;

        let today = Utc::now().date_naive();
        let (xp_awarded, level_before, stats) =
            update_user_progress(&*self.user_repository, &params.user_id, |user| {
                let level_before = user.stats.level();
                let xp_awarded = user.complete_session(today);
                (xp_awarded, level_before, user.stats)
            })
            .await?;

        let mut activities = vec![ActivityKind::SessionCompleted {
            session_id: session.id,
            session_name: session.name,
        }];
        activities.extend(level_up(level_before, stats.level()));
        publish_activities(&*self.activity_repository, &params.user_id, activities).await;

        Ok(SessionCompletion {
            session_id: params.session_id,
            xp_awarded,
            xp: stats.xp,
            level: stats.level(),
            sessions_done: stats.sessions_done,
        })
    }
}
