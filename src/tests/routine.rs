use std::sync::Arc;

use axum::http::StatusCode;

use crate::{
    application::{
        ports::repositories::FavRoutineRepository,
        use_cases::routines::{
            CreateRoutineUseCase, CreateRoutineUseCaseParams, DeleteRoutineUseCase,
            DeleteRoutineUseCaseParams, RoutineInput, UpdateRoutineUseCase,
            UpdateRoutineUseCaseParams,
        },
    },
    domain::{
        entities::{FavRoutine, Routine},
        value_objects::{DifficultyLevel, Error},
    },
    tests::support::{
        InMemoryFavRoutineRepository, InMemoryRoutineRepository, InMemorySessionRepository,
        InMemoryUserRepository, session, user,
    },
};

struct Context {
    users: Arc<InMemoryUserRepository>,
    sessions: Arc<InMemorySessionRepository>,
    routines: Arc<InMemoryRoutineRepository>,
    fav_routines: Arc<InMemoryFavRoutineRepository>,
}

fn input(session_ids: &[&str]) -> RoutineInput {
    RoutineInput {
        title: " PPL ".to_string(),
        level: DifficultyLevel::Medium,
        duration_weeks: 8,
        description: "Push pull legs".to_string(),
        session_ids: session_ids.iter().map(|id| id.to_string()).collect(),
    }
}

impl Context {
    fn new() -> Self {
        Self {
            users: Arc::new(InMemoryUserRepository::with(vec![user("u1", "alice")])),
            sessions: Arc::new(InMemorySessionRepository::with(vec![
                session("s1", "Push"),
                session("s2", "Pull"),
            ])),
            routines: Arc::default(),
            fav_routines: Arc::default(),
        }
    }

    async fn create(&self, input: RoutineInput) -> Result<Routine, Error> {
        CreateRoutineUseCase {
            routine_repository: self.routines.clone(),
            session_repository: self.sessions.clone(),
            user_repository: self.users.clone(),
        }
        .execute(CreateRoutineUseCaseParams {
            user_id: "u1".to_string(),
            routine: input,
        })
        .await
    }

    async fn update(&self, id: &str, input: RoutineInput) -> Result<Routine, Error> {
        UpdateRoutineUseCase {
            routine_repository: self.routines.clone(),
            session_repository: self.sessions.clone(),
        }
        .execute(UpdateRoutineUseCaseParams {
            id: id.to_string(),
            routine: input,
        })
        .await
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        DeleteRoutineUseCase {
            routine_repository: self.routines.clone(),
            fav_routine_repository: self.fav_routines.clone(),
        }
        .execute(DeleteRoutineUseCaseParams { id: id.to_string() })
        .await
    }
}

fn session_names(routine: &Routine) -> Vec<&str> {
    routine.sessions.iter().map(|s| s.name.as_str()).collect()
}

#[tokio::test]
async fn create_routine_resolves_sessions_in_order() {
    let context = Context::new();
    let routine = context.create(input(&["s2", "s1", "s2"])).await.unwrap();

    assert_eq!(routine.title, "PPL");
    assert_eq!(routine.author.user_id, "u1");
    assert_eq!(routine.author.user_name, "alice");
    assert_eq!(session_names(&routine), ["Pull", "Push", "Pull"]);
    assert_eq!(context.routines.get(&routine.id), Some(routine));
}

#[tokio::test]
async fn create_routine_validates_input() {
    let context = Context::new();
    let invalid = [
        RoutineInput {
            title: "  ".to_string(),
            ..input(&["s1"])
        },
        RoutineInput {
            duration_weeks: 0,
            ..input(&["s1"])
        },
        input(&[]),
        input(&["s1", "ghost"]),
    ];

    for routine in invalid {
        let error = context.create(routine).await.unwrap_err();
        assert_eq!(error.code, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn update_routine_replaces_fields() {
    let context = Context::new();
    let created = context.create(input(&["s1"])).await.unwrap();

    let updated = context
        .update(
            &created.id,
            RoutineInput {
                title: "Upper/Lower".to_string(),
                level: DifficultyLevel::Hard,
                duration_weeks: 12,
                ..input(&["s2"])
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.author, created.author);
    assert_eq!(updated.title, "Upper/Lower");
    assert_eq!(updated.level, DifficultyLevel::Hard);
    assert_eq!(updated.duration_weeks, 12);
    assert_eq!(session_names(&updated), ["Pull"]);
}

#[tokio::test]
async fn update_unknown_routine_is_not_found() {
    let context = Context::new();
    let error = context.update("ghost", input(&["s1"])).await.unwrap_err();
    assert_eq!(error.code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_routine_removes_it_and_its_favorites() {
    let context = Context::new();
    let routine = context.create(input(&["s1"])).await.unwrap();
    for (user_id, routine_id) in [
        ("u1", routine.id.as_str()),
        ("u2", routine.id.as_str()),
        ("u1", "other"),
    ] {
        let fav = FavRoutine {
            user_id: user_id.into(),
            routine_id: routine_id.into(),
        };
        context.fav_routines.save(&fav).await.unwrap();
    }

    context.delete(&routine.id).await.unwrap();

    let remaining: Vec<String> = context
        .fav_routines
        .all()
        .into_iter()
        .map(|f| f.routine_id)
        .collect();
    assert!(context.routines.get(&routine.id).is_none());
    assert_eq!(remaining, ["other"]);
    assert_eq!(
        context.delete(&routine.id).await.unwrap_err().code,
        StatusCode::NOT_FOUND
    );
}
