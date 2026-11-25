use std::sync::Arc;

use axum::http::StatusCode;

use crate::{
    application::{
        ports::repositories::FavSessionRepository,
        use_cases::sessions::{
            CompleteSessionUseCase, CompleteSessionUseCaseParams, CreateSessionUseCase,
            CreateSessionUseCaseParams, DeleteSessionUseCase, DeleteSessionUseCaseParams,
            SessionExerciseInput, UpdateSessionUseCase, UpdateSessionUseCaseParams,
        },
    },
    domain::{
        entities::{ActivityKind, FavSession, Session, xp_for_level},
        value_objects::{Error, Muscle},
    },
    tests::support::{
        InMemoryActivityRepository, InMemoryExerciseRepository, InMemoryFavSessionRepository,
        InMemorySessionRepository, InMemoryUserRepository, exercise, session, user,
    },
};

struct Context {
    users: Arc<InMemoryUserRepository>,
    sessions: Arc<InMemorySessionRepository>,
    exercises: Arc<InMemoryExerciseRepository>,
    fav_sessions: Arc<InMemoryFavSessionRepository>,
    activities: Arc<InMemoryActivityRepository>,
}

fn set(exercise_id: &str, sets: u16, reps: u16) -> SessionExerciseInput {
    SessionExerciseInput {
        exercise_id: exercise_id.to_string(),
        sets,
        reps,
        rest_secs: 90,
    }
}

impl Context {
    fn new() -> Self {
        Self {
            users: Arc::new(InMemoryUserRepository::with(vec![user("u1", "alice")])),
            sessions: Arc::new(InMemorySessionRepository::with(vec![session("s1", "Push")])),
            exercises: Arc::new(InMemoryExerciseRepository::with(vec![
                exercise("bench", "Bench", vec![Muscle::Chest, Muscle::Triceps]),
                exercise("dips", "Dips", vec![Muscle::Triceps, Muscle::Shoulders]),
            ])),
            fav_sessions: Arc::default(),
            activities: Arc::default(),
        }
    }

    async fn create(
        &self,
        name: &str,
        exercises: Vec<SessionExerciseInput>,
    ) -> Result<Session, Error> {
        CreateSessionUseCase {
            session_repository: self.sessions.clone(),
            exercise_repository: self.exercises.clone(),
            user_repository: self.users.clone(),
        }
        .execute(CreateSessionUseCaseParams {
            user_id: "u1".to_string(),
            name: name.to_string(),
            exercises,
        })
        .await
    }

    async fn update(
        &self,
        id: &str,
        name: &str,
        exercises: Vec<SessionExerciseInput>,
    ) -> Result<Session, Error> {
        UpdateSessionUseCase {
            session_repository: self.sessions.clone(),
            exercise_repository: self.exercises.clone(),
        }
        .execute(UpdateSessionUseCaseParams {
            id: id.to_string(),
            name: name.to_string(),
            exercises,
        })
        .await
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        DeleteSessionUseCase {
            session_repository: self.sessions.clone(),
            fav_session_repository: self.fav_sessions.clone(),
        }
        .execute(DeleteSessionUseCaseParams { id: id.to_string() })
        .await
    }

    fn complete_use_case(&self) -> CompleteSessionUseCase {
        CompleteSessionUseCase {
            session_repository: self.sessions.clone(),
            user_repository: self.users.clone(),
            activity_repository: self.activities.clone(),
        }
    }

    fn activity_kinds(&self) -> Vec<ActivityKind> {
        self.activities.all().into_iter().map(|a| a.kind).collect()
    }
}

fn completion(session_id: &str) -> CompleteSessionUseCaseParams {
    CompleteSessionUseCaseParams {
        user_id: "u1".to_string(),
        session_id: session_id.to_string(),
    }
}

#[tokio::test]
async fn create_session_builds_exercises_and_muscles_from_catalog() {
    let context = Context::new();
    let created = context
        .create(" Push day ", vec![set("dips", 3, 10), set("bench", 4, 8)])
        .await
        .unwrap();

    assert_eq!(created.name, "Push day");
    assert_eq!(created.author.user_id, "u1");
    assert_eq!(created.author.user_name, "alice");
    assert_eq!(created.exercises[0].name, "Dips");
    assert_eq!(
        (
            created.exercises[1].sets,
            created.exercises[1].reps,
            created.exercises[1].rest_secs
        ),
        (4, 8, 90)
    );
    assert_eq!(created.muscles.len(), 3);
    assert!(created.muscles.contains(&Muscle::Chest));
    assert_eq!(context.sessions.get(&created.id), Some(created));
}

#[tokio::test]
async fn create_session_validates_input() {
    let context = Context::new();
    let invalid = [
        ("   ", vec![set("bench", 3, 8)]),
        ("Push", vec![]),
        ("Push", vec![set("bench", 0, 8)]),
        ("Push", vec![set("bench", 3, 0)]),
        ("Push", vec![set("ghost", 3, 8)]),
    ];

    for (name, exercises) in invalid {
        let error = context.create(name, exercises).await.unwrap_err();
        assert_eq!(error.code, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn update_session_replaces_name_exercises_and_muscles() {
    let context = Context::new();
    let updated = context
        .update("s1", "Triceps", vec![set("dips", 3, 12)])
        .await
        .unwrap();

    assert_eq!(updated.name, "Triceps");
    assert_eq!(updated.exercises.len(), 1);
    assert_eq!(updated.muscles, [Muscle::Triceps, Muscle::Shoulders]);
    assert_eq!(context.sessions.get("s1").unwrap().name, "Triceps");
}

#[tokio::test]
async fn update_session_errors() {
    let context = Context::new();
    let missing = context
        .update("ghost", "Push", vec![set("bench", 3, 8)])
        .await;
    let unknown_exercise = context.update("s1", "Push", vec![set("ghost", 3, 8)]).await;

    assert_eq!(missing.unwrap_err().code, StatusCode::NOT_FOUND);
    assert_eq!(unknown_exercise.unwrap_err().code, StatusCode::BAD_REQUEST);
    assert_eq!(context.sessions.get("s1").unwrap().name, "Push");
}

#[tokio::test]
async fn delete_session_removes_it_and_its_favorites() {
    let context = Context::new();
    for (user_id, session_id) in [("u1", "s1"), ("u2", "s1"), ("u1", "s2")] {
        let fav = FavSession {
            user_id: user_id.into(),
            session_id: session_id.into(),
        };
        context.fav_sessions.save(&fav).await.unwrap();
    }

    context.delete("s1").await.unwrap();

    let remaining: Vec<String> = context
        .fav_sessions
        .all()
        .into_iter()
        .map(|f| f.session_id)
        .collect();
    assert!(context.sessions.get("s1").is_none());
    assert_eq!(remaining, ["s2"]);
    assert_eq!(
        context.delete("s1").await.unwrap_err().code,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn complete_session_awards_xp_and_publishes_activity() {
    let context = Context::new();
    let result = context
        .complete_use_case()
        .execute(completion("s1"))
        .await
        .unwrap();

    assert_eq!(result.session_id, "s1");
    assert_eq!(result.xp_awarded, 50);
    assert_eq!(result.xp, 50);
    assert_eq!(result.sessions_done, 1);
    assert_eq!(context.users.get("u1").unwrap().stats.sessions_done, 1);
    assert_eq!(
        context.activity_kinds(),
        [ActivityKind::SessionCompleted {
            session_id: "s1".to_string(),
            session_name: "Push".to_string(),
        }]
    );
}

#[tokio::test]
async fn complete_session_publishes_level_up_when_crossing_threshold() {
    let context = Context::new();
    let mut stored = context.users.get("u1").unwrap();
    stored.stats.xp = xp_for_level(1) - 1;
    context.users.insert(stored);

    let result = context
        .complete_use_case()
        .execute(completion("s1"))
        .await
        .unwrap();

    assert_eq!(result.level, 1);
    assert_eq!(
        context.activity_kinds()[1],
        ActivityKind::LevelUp { level: 1 }
    );
}

#[tokio::test]
async fn complete_unknown_session_is_not_found() {
    let context = Context::new();
    let result = context
        .complete_use_case()
        .execute(completion("ghost"))
        .await;

    assert_eq!(result.unwrap_err().code, StatusCode::NOT_FOUND);
    assert_eq!(context.users.get("u1").unwrap().stats.xp, 0);
    assert!(context.activities.all().is_empty());
}
