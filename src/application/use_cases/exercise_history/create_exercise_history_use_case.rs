use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::{
    application::{
        ports::repositories::{
            ExerciseHistoryRepository, ExerciseRepository, SessionRepository, UserRepository,
        },
        use_cases::exercise_history::user_progress::update_user_progress,
    },
    domain::{entities::ExerciseHistory, value_objects::Error},
};

const MAX_CLOCK_SKEW_MINUTES: i64 = 5;

pub struct CreateExerciseHistoryUseCase {
    pub exercise_history_repository: Arc<dyn ExerciseHistoryRepository>,
    pub exercise_repository: Arc<dyn ExerciseRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct CreateExerciseHistoryUseCaseParams {
    pub user_id: String,
    pub exercise_id: String,
    pub session_id: Option<String>,
    pub performed_at: Option<DateTime<Utc>>,
    pub weight_kg: Option<f32>,
    pub reps: Option<u16>,
}

impl CreateExerciseHistoryUseCase {
    pub async fn execute(
        &self,
        params: CreateExerciseHistoryUseCaseParams,
    ) -> Result<ExerciseHistory, Error> {
        let now = Utc::now();
        let performed_at = params.performed_at.unwrap_or(now);

        if performed_at > now + Duration::minutes(MAX_CLOCK_SKEW_MINUTES) {
            return Err(Error::bad_request("performed_at must not be in the future"));
        }

        if params
            .weight_kg
            .is_some_and(|weight| !weight.is_finite() || weight < 0.0)
        {
            return Err(Error::bad_request("weight_kg must be 0 or greater"));
        }

        if params.reps == Some(0) {
            return Err(Error::bad_request("reps must be greater than 0"));
        }

        self.exercise_repository
            .find_by_id(&params.exercise_id)
            .await?
            .ok_or_else(|| {
                Error::bad_request(&format!("Exercise {} does not exist", params.exercise_id))
            })?;

        if let Some(session_id) = &params.session_id {
            self.session_repository
                .find_by_id(session_id)
                .await?
                .ok_or_else(|| {
                    Error::bad_request(&format!("Session {session_id} does not exist"))
                })?;
        }

        let today = now.date_naive();
        let xp_awarded = update_user_progress(&*self.user_repository, &params.user_id, |user| {
            user.record_exercise(&params.exercise_id, params.weight_kg, params.reps, today)
        })
        .await?;

        let mut history = ExerciseHistory::new(
            params.user_id,
            params.exercise_id,
            params.session_id,
            performed_at,
            params.weight_kg,
            params.reps,
        );
        history.xp_awarded = xp_awarded;

        self.exercise_history_repository.save(&history).await?;

        Ok(history)
    }
}
