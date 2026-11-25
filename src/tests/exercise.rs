use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};

use crate::{
    application::use_cases::{
        exercise_history::{
            CreateExerciseHistoryUseCase, CreateExerciseHistoryUseCaseParams,
            DeleteExerciseHistoryUseCase, DeleteExerciseHistoryUseCaseParams,
        },
        exercises::{
            CreateExerciseUseCase, CreateExerciseUseCaseParams, DeleteExerciseUseCase,
            DeleteExerciseUseCaseParams, ExerciseInput, GetExerciseUseCase,
            GetExerciseUseCaseParams, ListExercisesUseCase, ListExercisesUseCaseParams,
            UpdateExerciseUseCase, UpdateExerciseUseCaseParams,
        },
    },
    domain::{
        entities::{ActivityKind, Exercise, ExerciseHistory, xp_for_level},
        value_objects::{Error, ExerciseCategory, Muscle, Page},
    },
    tests::support::{
        InMemoryActivityRepository, InMemoryExerciseHistoryRepository, InMemoryExerciseRepository,
        InMemorySessionRepository, InMemoryUserRepository, exercise, session, user,
    },
};

fn input(name: &str, muscles: Vec<Muscle>) -> ExerciseInput {
    ExerciseInput {
        name: name.to_string(),
        category: ExerciseCategory::Strength,
        primary_muscles: muscles,
        secondary_muscles: vec![],
        description: "Lie on a bench".to_string(),
        instructions: vec!["Lower the bar".to_string(), "Press".to_string()],
        video_url: None,
        img_url: Some("https://img.example.com/bench.png".to_string()),
    }
}

async fn create(
    repository: &Arc<InMemoryExerciseRepository>,
    input: ExerciseInput,
) -> Result<Exercise, Error> {
    CreateExerciseUseCase {
        exercise_repository: repository.clone(),
    }
    .execute(CreateExerciseUseCaseParams { exercise: input })
    .await
}

async fn update(
    repository: &Arc<InMemoryExerciseRepository>,
    id: &str,
    input: ExerciseInput,
) -> Result<Exercise, Error> {
    UpdateExerciseUseCase {
        exercise_repository: repository.clone(),
    }
    .execute(UpdateExerciseUseCaseParams {
        id: id.to_string(),
        exercise: input,
    })
    .await
}

async fn list(
    repository: &Arc<InMemoryExerciseRepository>,
    limit: Option<u16>,
    cursor: Option<String>,
    muscle: Option<Muscle>,
    category: Option<ExerciseCategory>,
) -> Result<Page<Exercise>, Error> {
    ListExercisesUseCase {
        exercise_repository: repository.clone(),
    }
    .execute(ListExercisesUseCaseParams {
        limit,
        cursor,
        muscle,
        category,
    })
    .await
}

#[tokio::test]
async fn create_exercise_trims_name_and_saves() {
    let repository = Arc::new(InMemoryExerciseRepository::default());
    let created = create(&repository, input("  Bench press ", vec![Muscle::Chest]))
        .await
        .unwrap();

    assert_eq!(created.name, "Bench press");
    assert_eq!(created.instructions.len(), 2);
    assert_eq!(repository.get(&created.id), Some(created));
}

#[tokio::test]
async fn create_exercise_validates_input() {
    let repository = Arc::new(InMemoryExerciseRepository::default());
    let no_name = create(&repository, input("  ", vec![Muscle::Chest])).await;
    let no_muscles = create(&repository, input("Bench", vec![])).await;

    assert_eq!(no_name.unwrap_err().code, StatusCode::BAD_REQUEST);
    assert_eq!(no_muscles.unwrap_err().code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_exercise_replaces_fields() {
    let repository = Arc::new(InMemoryExerciseRepository::with(vec![exercise(
        "bench",
        "Bench",
        vec![Muscle::Chest],
    )]));

    let updated = update(
        &repository,
        "bench",
        input("Incline bench", vec![Muscle::Chest, Muscle::Shoulders]),
    )
    .await
    .unwrap();

    assert_eq!(updated.id, "bench");
    assert_eq!(updated.name, "Incline bench");
    assert_eq!(repository.get("bench").unwrap().primary_muscles.len(), 2);
}

#[tokio::test]
async fn exercises_keep_their_category() {
    let repository = Arc::new(InMemoryExerciseRepository::default());
    let created = create(
        &repository,
        ExerciseInput {
            category: ExerciseCategory::Cardio,
            ..input("Running", vec![Muscle::Quadriceps, Muscle::Calves])
        },
    )
    .await
    .unwrap();

    assert_eq!(created.category, ExerciseCategory::Cardio);
    assert_eq!(
        repository.get(&created.id).unwrap().category,
        ExerciseCategory::Cardio
    );

    let updated = update(
        &repository,
        &created.id,
        ExerciseInput {
            category: ExerciseCategory::Mobility,
            ..input("Running drills", vec![Muscle::Calves])
        },
    )
    .await
    .unwrap();

    assert_eq!(updated.category, ExerciseCategory::Mobility);
}

#[test]
fn legacy_exercise_defaults_to_strength_with_muscles_as_primary() {
    assert_eq!(
        serde_json::to_string(&ExerciseCategory::Cardio).unwrap(),
        "\"cardio\""
    );

    let legacy: Exercise = serde_json::from_value(serde_json::json!({
        "id": "bench",
        "name": "Bench",
        "muscles": ["chest"],
        "description": "",
        "instructions": [],
        "video_url": null,
        "img_url": null,
    }))
    .unwrap();

    assert_eq!(legacy.category, ExerciseCategory::Strength);
    assert_eq!(legacy.primary_muscles, [Muscle::Chest]);
    assert!(legacy.secondary_muscles.is_empty());
}

#[tokio::test]
async fn update_exercise_errors() {
    let repository = Arc::new(InMemoryExerciseRepository::with(vec![exercise(
        "bench",
        "Bench",
        vec![Muscle::Chest],
    )]));

    let missing = update(&repository, "ghost", input("Bench", vec![Muscle::Chest])).await;
    let invalid = update(&repository, "bench", input("Bench", vec![])).await;

    assert_eq!(missing.unwrap_err().code, StatusCode::NOT_FOUND);
    assert_eq!(invalid.unwrap_err().code, StatusCode::BAD_REQUEST);
    assert_eq!(
        repository.get("bench").unwrap().primary_muscles,
        [Muscle::Chest]
    );
}

#[tokio::test]
async fn get_and_delete_exercise() {
    let repository = Arc::new(InMemoryExerciseRepository::with(vec![exercise(
        "bench",
        "Bench",
        vec![Muscle::Chest],
    )]));
    let get = GetExerciseUseCase {
        exercise_repository: repository.clone(),
    };
    let delete = DeleteExerciseUseCase {
        exercise_repository: repository.clone(),
    };

    let found = get
        .execute(GetExerciseUseCaseParams { id: "bench".into() })
        .await
        .unwrap();
    assert_eq!(found.name, "Bench");

    delete
        .execute(DeleteExerciseUseCaseParams { id: "bench".into() })
        .await
        .unwrap();

    let after_delete = get
        .execute(GetExerciseUseCaseParams { id: "bench".into() })
        .await;
    let delete_again = delete
        .execute(DeleteExerciseUseCaseParams { id: "bench".into() })
        .await;

    assert_eq!(after_delete.unwrap_err().code, StatusCode::NOT_FOUND);
    assert_eq!(delete_again.unwrap_err().code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_exercises_filters_and_paginates() {
    let repository = Arc::new(InMemoryExerciseRepository::with(vec![
        exercise("a", "Bench", vec![Muscle::Chest]),
        exercise("b", "Squat", vec![Muscle::Quadriceps]),
        exercise("c", "Dips", vec![Muscle::Chest, Muscle::Triceps]),
    ]));

    let first = list(&repository, Some(1), None, Some(Muscle::Chest), None)
        .await
        .unwrap();
    let second = list(
        &repository,
        Some(1),
        first.next_cursor.clone(),
        Some(Muscle::Chest),
        None,
    )
    .await
    .unwrap();

    assert_eq!(first.items[0].id, "a");
    assert_eq!(second.items[0].id, "c");
    assert_eq!(second.next_cursor, None);
}

#[tokio::test]
async fn list_exercises_validates_limit() {
    let repository = Arc::new(InMemoryExerciseRepository::default());

    assert!(list(&repository, None, None, None, None).await.is_ok());
    assert_eq!(
        list(&repository, Some(0), None, None, None)
            .await
            .unwrap_err()
            .code,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        list(&repository, Some(101), None, None, None)
            .await
            .unwrap_err()
            .code,
        StatusCode::BAD_REQUEST
    );
}

struct HistoryContext {
    users: Arc<InMemoryUserRepository>,
    history: Arc<InMemoryExerciseHistoryRepository>,
    activities: Arc<InMemoryActivityRepository>,
}

struct Entry {
    exercise_id: &'static str,
    session_id: Option<&'static str>,
    performed_at: Option<DateTime<Utc>>,
    weight_kg: Option<f32>,
    reps: Option<u16>,
}

fn entry(weight_kg: Option<f32>, reps: Option<u16>) -> Entry {
    Entry {
        exercise_id: "bench",
        session_id: None,
        performed_at: None,
        weight_kg,
        reps,
    }
}

impl HistoryContext {
    fn new() -> Self {
        Self {
            users: Arc::new(InMemoryUserRepository::with(vec![user("u1", "alice")])),
            history: Arc::default(),
            activities: Arc::default(),
        }
    }

    async fn create(&self, entry: Entry) -> Result<ExerciseHistory, Error> {
        let exercises = InMemoryExerciseRepository::with(vec![exercise(
            "bench",
            "Bench press",
            vec![Muscle::Chest],
        )]);
        let sessions = InMemorySessionRepository::with(vec![session("s1", "Push")]);

        CreateExerciseHistoryUseCase {
            exercise_history_repository: self.history.clone(),
            exercise_repository: Arc::new(exercises),
            session_repository: Arc::new(sessions),
            user_repository: self.users.clone(),
            activity_repository: self.activities.clone(),
        }
        .execute(CreateExerciseHistoryUseCaseParams {
            user_id: "u1".to_string(),
            exercise_id: entry.exercise_id.to_string(),
            session_id: entry.session_id.map(str::to_string),
            performed_at: entry.performed_at,
            weight_kg: entry.weight_kg,
            reps: entry.reps,
        })
        .await
    }

    async fn delete(&self, user_id: &str, id: &str) -> Result<(), Error> {
        DeleteExerciseHistoryUseCase {
            exercise_history_repository: self.history.clone(),
            user_repository: self.users.clone(),
        }
        .execute(DeleteExerciseHistoryUseCaseParams {
            user_id: user_id.to_string(),
            id: id.to_string(),
        })
        .await
    }

    fn activity_kinds(&self) -> Vec<ActivityKind> {
        self.activities.all().into_iter().map(|a| a.kind).collect()
    }
}

async fn rejected(entry: Entry) -> StatusCode {
    let context = HistoryContext::new();
    let code = context.create(entry).await.unwrap_err().code;

    assert!(context.history.all().is_empty());
    assert_eq!(context.users.get("u1").unwrap().stats.xp, 0);
    code
}

#[tokio::test]
async fn history_rejects_invalid_entries_without_side_effects() {
    let future = Some(Utc::now() + Duration::minutes(10));
    let invalid = [
        Entry {
            performed_at: future,
            ..entry(None, Some(5))
        },
        entry(Some(-1.0), Some(5)),
        entry(Some(f32::NAN), Some(5)),
        entry(Some(f32::INFINITY), Some(5)),
        entry(None, Some(0)),
        Entry {
            exercise_id: "ghost",
            ..entry(None, Some(5))
        },
        Entry {
            session_id: Some("ghost"),
            ..entry(None, Some(5))
        },
    ];

    for entry in invalid {
        assert_eq!(rejected(entry).await, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn history_accepts_small_clock_skew_and_past_dates() {
    let context = HistoryContext::new();

    for performed_at in [
        Utc::now() + Duration::minutes(2),
        Utc::now() - Duration::days(3),
    ] {
        let history = context
            .create(Entry {
                performed_at: Some(performed_at),
                ..entry(None, Some(5))
            })
            .await
            .unwrap();
        assert_eq!(history.performed_at, performed_at);
    }
}

#[tokio::test]
async fn history_saves_entry_and_awards_xp() {
    let context = HistoryContext::new();
    let history = context
        .create(Entry {
            session_id: Some("s1"),
            ..entry(Some(60.0), Some(8))
        })
        .await
        .unwrap();

    let stored = context.users.get("u1").unwrap();
    assert_eq!(history.xp_awarded, 10);
    assert_eq!(history.session_id.as_deref(), Some("s1"));
    assert_eq!(context.history.all(), vec![history]);
    assert_eq!(stored.stats.xp, 10);
    assert_eq!(stored.stats.exercises_done, 1);
    assert!(context.activities.all().is_empty());
}

#[tokio::test]
async fn history_personal_record_publishes_activity() {
    let context = HistoryContext::new();
    context.create(entry(Some(60.0), Some(8))).await.unwrap();
    let history = context.create(entry(Some(70.0), Some(6))).await.unwrap();

    assert_eq!(history.xp_awarded, 35);
    assert_eq!(
        context.activity_kinds(),
        [ActivityKind::PersonalRecord {
            exercise_id: "bench".to_string(),
            exercise_name: "Bench press".to_string(),
            weight_kg: 70.0,
            reps: 6,
        }]
    );
}

#[tokio::test]
async fn history_level_up_publishes_activity() {
    let context = HistoryContext::new();
    let mut stored = context.users.get("u1").unwrap();
    stored.stats.xp = xp_for_level(1) - 1;
    context.users.insert(stored);

    context.create(entry(None, None)).await.unwrap();

    assert_eq!(
        context.activity_kinds(),
        [ActivityKind::LevelUp { level: 1 }]
    );
}

#[tokio::test]
async fn history_delete_removes_entry_and_reverts_xp() {
    let context = HistoryContext::new();
    let first = context.create(entry(None, Some(5))).await.unwrap();
    context.create(entry(None, Some(5))).await.unwrap();

    context.delete("u1", &first.id).await.unwrap();

    let stored = context.users.get("u1").unwrap();
    assert_eq!(context.history.all().len(), 1);
    assert_eq!(stored.stats.xp, 10);
    assert_eq!(stored.stats.exercises_done, 1);
}

#[tokio::test]
async fn history_cannot_delete_another_users_entry() {
    let context = HistoryContext::new();
    context.users.insert(user("u2", "bob"));
    let history = context.create(entry(None, Some(5))).await.unwrap();

    let error = context.delete("u2", &history.id).await.unwrap_err();

    assert_eq!(error.code, StatusCode::NOT_FOUND);
    assert_eq!(context.history.all().len(), 1);
    assert_eq!(context.users.get("u1").unwrap().stats.xp, 10);
}

#[tokio::test]
async fn list_exercises_filters_by_category_and_muscle() {
    let mut running = exercise("b", "Running", vec![Muscle::Quadriceps]);
    running.category = ExerciseCategory::Cardio;
    let mut rowing = exercise("c", "Rowing", vec![Muscle::UpperBack]);
    rowing.category = ExerciseCategory::Cardio;
    let repository = Arc::new(InMemoryExerciseRepository::with(vec![
        exercise("a", "Squat", vec![Muscle::Quadriceps]),
        running,
        rowing,
    ]));

    let ids = |page: Page<Exercise>| page.items.into_iter().map(|e| e.id).collect::<Vec<_>>();

    let cardio = list(
        &repository,
        None,
        None,
        None,
        Some(ExerciseCategory::Cardio),
    )
    .await
    .unwrap();
    let cardio_legs = list(
        &repository,
        None,
        None,
        Some(Muscle::Quadriceps),
        Some(ExerciseCategory::Cardio),
    )
    .await
    .unwrap();
    let strength = list(
        &repository,
        None,
        None,
        None,
        Some(ExerciseCategory::Strength),
    )
    .await
    .unwrap();

    assert_eq!(ids(cardio), ["b", "c"]);
    assert_eq!(ids(cardio_legs), ["b"]);
    assert_eq!(ids(strength), ["a"]);
}

#[tokio::test]
async fn create_exercise_stores_primary_and_secondary_muscles() {
    let repository = Arc::new(InMemoryExerciseRepository::default());
    let created = create(
        &repository,
        ExerciseInput {
            secondary_muscles: vec![Muscle::Triceps, Muscle::Shoulders],
            ..input("Bench", vec![Muscle::Chest])
        },
    )
    .await
    .unwrap();

    let stored = repository.get(&created.id).unwrap();
    assert_eq!(stored.primary_muscles, [Muscle::Chest]);
    assert_eq!(
        stored.secondary_muscles,
        [Muscle::Triceps, Muscle::Shoulders]
    );
}

#[tokio::test]
async fn exercise_muscles_must_not_repeat() {
    let repository = Arc::new(InMemoryExerciseRepository::default());
    let invalid = [
        ExerciseInput {
            secondary_muscles: vec![Muscle::Chest],
            ..input("Bench", vec![Muscle::Chest])
        },
        input("Bench", vec![Muscle::Chest, Muscle::Chest]),
        ExerciseInput {
            secondary_muscles: vec![Muscle::Triceps, Muscle::Triceps],
            ..input("Bench", vec![Muscle::Chest])
        },
        ExerciseInput {
            secondary_muscles: vec![Muscle::Triceps],
            ..input("Bench", vec![])
        },
    ];

    for exercise in invalid {
        let error = create(&repository, exercise).await.unwrap_err();
        assert_eq!(error.code, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn list_exercises_muscle_filter_matches_secondary_muscles() {
    let mut bench = exercise("a", "Bench", vec![Muscle::Chest]);
    bench.secondary_muscles = vec![Muscle::Triceps];
    let repository = Arc::new(InMemoryExerciseRepository::with(vec![
        bench,
        exercise("b", "Pushdown", vec![Muscle::Triceps]),
        exercise("c", "Squat", vec![Muscle::Quadriceps]),
    ]));

    let page = list(&repository, None, None, Some(Muscle::Triceps), None)
        .await
        .unwrap();

    let ids: Vec<String> = page.items.into_iter().map(|e| e.id).collect();
    assert_eq!(ids, ["a", "b"]);
}
