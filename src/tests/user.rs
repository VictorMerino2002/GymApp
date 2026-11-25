use std::sync::{Arc, atomic::Ordering};

use axum::http::StatusCode;
use chrono::{NaiveDate, Utc};

use crate::{
    application::{
        ports::{
            JwtEncoderPort,
            repositories::{
                ActivityRepository, ExerciseHistoryRepository, FavRoutineRepository,
                FavSessionRepository,
            },
        },
        use_cases::{
            auth::GenerateAuthUseCaseResponse,
            me::{
                ChangePasswordUseCase, ChangePasswordUseCaseParams, DeleteAccountUseCase,
                DeleteAccountUseCaseParams, GetProfileUseCase, GetProfileUseCaseParams, Profile,
                UpdateProfileUseCase, UpdateProfileUseCaseParams,
            },
            user_progress::update_user_progress,
        },
    },
    domain::{
        entities::{
            Activity, ActivityKind, ExerciseHistory, ExerciseReward, FavRoutine, FavSession,
            PersonalRecord, Stats, User, level_from_xp, validate_password, validate_picture_url,
            validate_username, xp_for_level,
        },
        value_objects::{Error, Role},
    },
    tests::support::{
        FakePasswordHasher, InMemoryActivityRepository, InMemoryExerciseHistoryRepository,
        InMemoryFavRoutineRepository, InMemoryFavSessionRepository, InMemoryFriendshipRepository,
        InMemoryUserRepository, PASSWORD, jwt_encoder, user,
    },
};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn monday() -> NaiveDate {
    date(2024, 1, 1)
}

fn new_user() -> User {
    User::new("alice", "hash")
}

#[test]
fn new_user_starts_empty() {
    let user = new_user();
    assert_eq!(user.stats, Stats::default());
    assert_eq!(user.role, Role::User);
    assert_eq!(user.token_version, 0);
    assert_eq!(user.password_hash.as_deref(), Some("hash"));
    assert!(user.personal_records.is_empty());
}

#[test]
fn username_validation() {
    let longest = "a".repeat(20);
    let too_long = "a".repeat(21);

    for valid in ["abc", "alice_99", "a.b_c", "1user", longest.as_str()] {
        assert!(validate_username(valid).is_ok(), "{valid} should be valid");
    }

    let invalid = [
        "ab",
        too_long.as_str(),
        "bad name",
        "_alice",
        ".alice",
        "ñandú",
        "al-ice",
    ];
    for name in invalid {
        let error = validate_username(name).unwrap_err();
        assert_eq!(
            error.code,
            StatusCode::BAD_REQUEST,
            "{name} should be invalid"
        );
    }
}

#[test]
fn password_validation() {
    assert!(validate_password(&"a".repeat(8)).is_ok());
    assert!(validate_password(&"a".repeat(128)).is_ok());
    assert!(validate_password(&"a".repeat(7)).is_err());
    assert!(validate_password(&"a".repeat(129)).is_err());
}

#[test]
fn picture_url_validation() {
    assert!(validate_picture_url("https://cdn.example.com/a.png").is_ok());
    assert!(validate_picture_url("http://cdn.example.com/a.png").is_err());
    assert!(validate_picture_url("https://cdn.example.com/a b.png").is_err());
    assert!(validate_picture_url(&format!("https://{}", "a".repeat(2048))).is_err());
}

#[test]
fn level_thresholds_match_xp_for_level() {
    assert_eq!(level_from_xp(0), 0);
    for level in 1..200 {
        let threshold = xp_for_level(level);
        assert_eq!(level_from_xp(threshold), level);
        assert_eq!(level_from_xp(threshold - 1), level - 1);
    }
}

#[test]
fn first_exercise_awards_set_xp_and_stores_record_without_pr() {
    let mut user = new_user();
    let reward = user.record_exercise("bench", Some(60.0), Some(8), monday());

    assert_eq!(
        reward,
        ExerciseReward {
            xp: 10,
            personal_record: false
        }
    );
    assert_eq!(user.stats.xp, 10);
    assert_eq!(user.stats.exercises_done, 1);
    assert_eq!(user.stats.streak_weeks, 1);
    assert_eq!(user.stats.last_workout_date, Some(monday()));
    assert_eq!(
        user.personal_records.get("bench"),
        Some(&PersonalRecord {
            weight_kg: 60.0,
            reps: 8
        })
    );
}

#[test]
fn heavier_weight_is_a_personal_record() {
    let mut user = new_user();
    user.record_exercise("bench", Some(60.0), Some(8), monday());
    let reward = user.record_exercise("bench", Some(62.5), Some(5), monday());

    assert_eq!(
        reward,
        ExerciseReward {
            xp: 35,
            personal_record: true
        }
    );
    assert_eq!(
        user.personal_records.get("bench"),
        Some(&PersonalRecord {
            weight_kg: 62.5,
            reps: 5
        })
    );
}

#[test]
fn same_weight_needs_more_reps_to_be_a_record() {
    let mut user = new_user();
    user.record_exercise("bench", Some(60.0), Some(8), monday());

    assert!(
        !user
            .record_exercise("bench", Some(60.0), Some(8), monday())
            .personal_record
    );
    assert!(
        !user
            .record_exercise("bench", Some(55.0), Some(12), monday())
            .personal_record
    );
    assert!(
        user.record_exercise("bench", Some(60.0), Some(9), monday())
            .personal_record
    );
}

#[test]
fn bodyweight_exercises_compare_reps() {
    let mut user = new_user();
    user.record_exercise("pullup", None, Some(8), monday());

    assert!(
        user.record_exercise("pullup", None, Some(10), monday())
            .personal_record
    );
    assert_eq!(
        user.personal_records.get("pullup"),
        Some(&PersonalRecord {
            weight_kg: 0.0,
            reps: 10
        })
    );
}

#[test]
fn exercise_without_reps_never_touches_records() {
    let mut user = new_user();
    let reward = user.record_exercise("plank", Some(10.0), None, monday());

    assert!(!reward.personal_record);
    assert!(user.personal_records.is_empty());
    assert_eq!(user.stats.exercises_done, 1);
}

#[test]
fn records_are_tracked_per_exercise() {
    let mut user = new_user();
    user.record_exercise("bench", Some(100.0), Some(5), monday());

    assert!(
        !user
            .record_exercise("squat", Some(50.0), Some(5), monday())
            .personal_record
    );
    assert_eq!(user.personal_records.len(), 2);
}

#[test]
fn workouts_in_the_same_week_keep_the_streak() {
    let mut user = new_user();
    user.record_exercise("bench", None, None, monday());
    let reward = user.record_exercise("bench", None, None, date(2024, 1, 7));

    assert_eq!(user.stats.streak_weeks, 1);
    assert_eq!(reward.xp, 10);
}

#[test]
fn consecutive_weeks_grow_the_streak_and_award_a_bonus_once_per_day() {
    let mut user = new_user();
    user.record_exercise("bench", None, None, date(2024, 1, 7));

    let first = user.record_exercise("bench", None, None, date(2024, 1, 8));
    let second = user.record_exercise("bench", None, None, date(2024, 1, 8));

    assert_eq!(user.stats.streak_weeks, 2);
    assert_eq!(first.xp, 15);
    assert_eq!(second.xp, 10);
}

#[test]
fn missing_a_week_resets_the_streak() {
    let mut user = new_user();
    user.record_exercise("bench", None, None, monday());
    user.record_exercise("bench", None, None, date(2024, 1, 8));
    assert_eq!(user.stats.streak_weeks, 2);

    user.record_exercise("bench", None, None, date(2024, 1, 22));
    assert_eq!(user.stats.streak_weeks, 1);
}

#[test]
fn streak_bonus_is_capped() {
    let mut user = new_user();
    user.stats.streak_weeks = 30;
    user.stats.last_workout_date = Some(date(2023, 12, 31));

    let reward = user.record_exercise("bench", None, None, monday());

    assert_eq!(user.stats.streak_weeks, 31);
    assert_eq!(reward.xp, 10 + 50);
}

#[test]
fn daily_xp_is_capped_and_resets_the_next_day() {
    let mut user = new_user();
    user.record_exercise("bench", None, None, monday());
    user.stats.xp_today = 296;
    let xp_before = user.stats.xp;

    assert_eq!(user.record_exercise("bench", None, None, monday()).xp, 4);
    assert_eq!(user.record_exercise("bench", None, None, monday()).xp, 0);
    assert_eq!(user.complete_session(monday()), 0);
    assert_eq!(user.stats.xp, xp_before + 4);
    assert_eq!(user.stats.xp_today, 300);
    assert_eq!(user.stats.exercises_done, 3);
    assert_eq!(user.stats.sessions_done, 1);

    assert_eq!(
        user.record_exercise("bench", None, None, date(2024, 1, 2))
            .xp,
        10
    );
    assert_eq!(user.stats.xp_today, 10);
}

#[test]
fn completing_a_session_awards_session_xp() {
    let mut user = new_user();

    assert_eq!(user.complete_session(monday()), 50);
    assert_eq!(user.stats.sessions_done, 1);
    assert_eq!(user.stats.xp, 50);
    assert_eq!(user.stats.last_workout_date, Some(monday()));
}

#[test]
fn undo_exercise_subtracts_without_underflow() {
    let mut user = new_user();
    user.record_exercise("bench", None, None, monday());

    user.undo_exercise(10);
    assert_eq!(user.stats.xp, 0);
    assert_eq!(user.stats.exercises_done, 0);

    user.undo_exercise(1000);
    assert_eq!(user.stats.xp, 0);
    assert_eq!(user.stats.exercises_done, 0);
}

#[tokio::test]
async fn progress_update_applies_change_and_bumps_version() {
    let repository = InMemoryUserRepository::with(vec![user("u1", "alice")]);

    let result = update_user_progress(&repository, "u1", |user| {
        user.stats.xp += 10;
        user.stats.xp
    })
    .await
    .unwrap();

    let stored = repository.get("u1").unwrap();
    assert_eq!(result, 10);
    assert_eq!(stored.stats.xp, 10);
    assert_eq!(stored.stats.progress_version, 1);
}

#[tokio::test]
async fn progress_update_retries_on_conflict_with_fresh_user() {
    let repository = InMemoryUserRepository::with(vec![user("u1", "alice")]);
    repository.progress_conflicts.store(2, Ordering::SeqCst);
    let mut calls = 0;

    update_user_progress(&repository, "u1", |user| {
        calls += 1;
        user.stats.xp += 10;
    })
    .await
    .unwrap();

    assert_eq!(calls, 3);
    assert_eq!(repository.get("u1").unwrap().stats.xp, 10);
}

#[tokio::test]
async fn progress_update_gives_up_after_three_attempts() {
    let repository = InMemoryUserRepository::with(vec![user("u1", "alice")]);
    repository.progress_conflicts.store(3, Ordering::SeqCst);

    let error = update_user_progress(&repository, "u1", |user| user.stats.xp += 10)
        .await
        .unwrap_err();

    assert_eq!(error.code, StatusCode::CONFLICT);
    assert_eq!(repository.get("u1").unwrap().stats.xp, 0);
}

#[tokio::test]
async fn progress_update_for_missing_user_is_unauthorized() {
    let repository = InMemoryUserRepository::default();
    let error = update_user_progress(&repository, "ghost", |_| ())
        .await
        .unwrap_err();
    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
}

fn users() -> Arc<InMemoryUserRepository> {
    Arc::new(InMemoryUserRepository::with(vec![
        user("u1", "alice"),
        user("u2", "bob"),
    ]))
}

#[tokio::test]
async fn get_profile_reports_level_progress() {
    let users = users();
    let mut stored = users.get("u1").unwrap();
    stored.stats.xp = xp_for_level(2) + 1;
    users.insert(stored);

    let profile = GetProfileUseCase {
        user_repository: users.clone(),
    }
    .execute(GetProfileUseCaseParams {
        user_id: "u1".to_string(),
    })
    .await
    .unwrap();

    assert_eq!(profile.username, "alice");
    assert_eq!(profile.stats.level, 2);
    assert_eq!(profile.stats.current_level_xp, xp_for_level(2));
    assert_eq!(profile.stats.next_level_xp, xp_for_level(3));
}

async fn change_password(
    users: &Arc<InMemoryUserRepository>,
    current: &str,
    new: &str,
) -> Result<GenerateAuthUseCaseResponse, Error> {
    ChangePasswordUseCase {
        user_repository: users.clone(),
        password_hasher: Arc::new(FakePasswordHasher::default()),
        jwt_encoder: jwt_encoder(),
    }
    .execute(ChangePasswordUseCaseParams {
        user_id: "u1".to_string(),
        current_password: current.to_string(),
        new_password: new.to_string(),
    })
    .await
}

#[tokio::test]
async fn change_password_stores_new_hash_and_bumps_token_version() {
    let users = users();
    let tokens = change_password(&users, PASSWORD, "new-password")
        .await
        .unwrap();

    let stored = users.get("u1").unwrap();
    assert_eq!(stored.password_hash.as_deref(), Some("hash:new-password"));
    assert_eq!(stored.token_version, 1);
    assert_eq!(
        jwt_encoder().decode(&tokens.access).unwrap().token_version,
        1
    );
}

#[tokio::test]
async fn change_password_requires_current_password() {
    let users = users();
    let error = change_password(&users, "wrong-password", "new-password")
        .await
        .err()
        .unwrap();

    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
    assert_eq!(users.get("u1").unwrap().token_version, 0);
}

#[tokio::test]
async fn change_password_validates_new_password() {
    let error = change_password(&users(), PASSWORD, "short")
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, StatusCode::BAD_REQUEST);
}

async fn update_profile(
    users: &Arc<InMemoryUserRepository>,
    username: Option<&str>,
    picture_url: Option<Option<&str>>,
) -> Result<Profile, Error> {
    UpdateProfileUseCase {
        user_repository: users.clone(),
    }
    .execute(UpdateProfileUseCaseParams {
        user_id: "u1".to_string(),
        username: username.map(str::to_string),
        picture_url: picture_url.map(|url| url.map(str::to_string)),
    })
    .await
}

#[tokio::test]
async fn update_profile_changes_only_given_fields() {
    let users = users();
    let picture = "https://img.example.com/a.png";

    update_profile(&users, None, Some(Some(picture)))
        .await
        .unwrap();
    let renamed = update_profile(&users, Some("alice_2"), None).await.unwrap();

    assert_eq!(renamed.username, "alice_2");
    assert_eq!(renamed.picture_url.as_deref(), Some(picture));

    let cleared = update_profile(&users, None, Some(None)).await.unwrap();
    assert_eq!(cleared.picture_url, None);
    assert_eq!(users.get("u1").unwrap().picture_url, None);
}

#[tokio::test]
async fn update_profile_validates_input() {
    let users = users();
    let bad_username = update_profile(&users, Some("x"), None).await.unwrap_err();
    let bad_picture = update_profile(&users, None, Some(Some("http://insecure")))
        .await
        .unwrap_err();

    assert_eq!(bad_username.code, StatusCode::BAD_REQUEST);
    assert_eq!(bad_picture.code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_profile_rejects_taken_username() {
    let users = users();
    let error = update_profile(&users, Some("Bob"), None).await.unwrap_err();

    assert_eq!(error.code, StatusCode::CONFLICT);
    assert_eq!(users.get("u1").unwrap().username, "alice");
}

struct DeleteContext {
    users: Arc<InMemoryUserRepository>,
    friendships: Arc<InMemoryFriendshipRepository>,
    fav_routines: Arc<InMemoryFavRoutineRepository>,
    fav_sessions: Arc<InMemoryFavSessionRepository>,
    history: Arc<InMemoryExerciseHistoryRepository>,
    activities: Arc<InMemoryActivityRepository>,
}

impl DeleteContext {
    async fn new() -> Self {
        let context = Self {
            users: users(),
            friendships: Arc::default(),
            fav_routines: Arc::default(),
            fav_sessions: Arc::default(),
            history: Arc::default(),
            activities: Arc::default(),
        };
        context.friendships.befriend("u1", "u2");

        for user_id in ["u1", "u2"] {
            let fav_routine = FavRoutine {
                user_id: user_id.into(),
                routine_id: "r1".into(),
            };
            let fav_session = FavSession {
                user_id: user_id.into(),
                session_id: "s1".into(),
            };
            let history = ExerciseHistory::new(user_id, "bench", None, Utc::now(), None, Some(5));
            let activity = Activity::new(user_id, ActivityKind::LevelUp { level: 1 });

            context.fav_routines.save(&fav_routine).await.unwrap();
            context.fav_sessions.save(&fav_session).await.unwrap();
            context.history.save(&history).await.unwrap();
            context.activities.save(&activity).await.unwrap();
        }

        context
    }

    async fn delete(&self, password: &str) -> Result<(), Error> {
        DeleteAccountUseCase {
            user_repository: self.users.clone(),
            friendship_repository: self.friendships.clone(),
            fav_routine_repository: self.fav_routines.clone(),
            fav_session_repository: self.fav_sessions.clone(),
            exercise_history_repository: self.history.clone(),
            activity_repository: self.activities.clone(),
            password_hasher: Arc::new(FakePasswordHasher::default()),
        }
        .execute(DeleteAccountUseCaseParams {
            user_id: "u1".to_string(),
            password: password.to_string(),
        })
        .await
    }
}

#[tokio::test]
async fn delete_account_removes_only_that_users_data() {
    let context = DeleteContext::new().await;
    context.delete(PASSWORD).await.unwrap();

    assert!(context.users.get("u1").is_none());
    assert!(context.users.get("u2").is_some());
    assert_eq!(context.friendships.count(), 0);

    let fav_routine_owners: Vec<String> = context
        .fav_routines
        .all()
        .into_iter()
        .map(|f| f.user_id)
        .collect();
    let fav_session_owners: Vec<String> = context
        .fav_sessions
        .all()
        .into_iter()
        .map(|f| f.user_id)
        .collect();
    let history_owners: Vec<String> = context
        .history
        .all()
        .into_iter()
        .map(|h| h.user_id)
        .collect();
    let activity_owners: Vec<String> = context
        .activities
        .all()
        .into_iter()
        .map(|a| a.user_id)
        .collect();

    assert_eq!(fav_routine_owners, ["u2"]);
    assert_eq!(fav_session_owners, ["u2"]);
    assert_eq!(history_owners, ["u2"]);
    assert_eq!(activity_owners, ["u2"]);
}

#[tokio::test]
async fn delete_account_requires_password() {
    let context = DeleteContext::new().await;
    let error = context.delete("wrong-password").await.unwrap_err();

    assert_eq!(error.code, StatusCode::UNAUTHORIZED);
    assert!(context.users.get("u1").is_some());
    assert_eq!(context.friendships.count(), 1);
    assert_eq!(context.history.all().len(), 2);
}
