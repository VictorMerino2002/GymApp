use std::sync::Arc;

use axum::http::StatusCode;
use chrono::Utc;

use crate::{
    application::{
        ports::repositories::{ActivityRepository, FriendshipRepository},
        use_cases::{
            activity_publisher::{level_up, publish_activities},
            social::{
                GetFeedUseCase, GetFeedUseCaseParams, GetLeaderboardUseCase,
                GetLeaderboardUseCaseParams, GetUserProfileUseCase, GetUserProfileUseCaseParams,
            },
        },
    },
    domain::entities::{Activity, ActivityKind, FriendshipStatus, PersonalRecord},
    tests::support::{
        InMemoryActivityRepository, InMemoryFriendshipRepository, InMemoryUserRepository, user,
    },
};

struct Context {
    users: Arc<InMemoryUserRepository>,
    friendships: Arc<InMemoryFriendshipRepository>,
    activities: Arc<InMemoryActivityRepository>,
}

impl Context {
    fn new() -> Self {
        let users = InMemoryUserRepository::default();
        for (id, username, xp) in [
            ("me", "carol", 100),
            ("f1", "dave", 300),
            ("f2", "Bob", 100),
            ("p1", "eve", 1000),
        ] {
            let mut stored = user(id, username);
            stored.stats.xp = xp;
            users.insert(stored);
        }

        let context = Self {
            users: Arc::new(users),
            friendships: Arc::default(),
            activities: Arc::default(),
        };
        context.friendships.befriend("me", "f1");
        context.friendships.befriend("f2", "me");
        context.friendships.request("p1", "me");
        context
    }

    async fn add_activity(&self, id: &str, user_id: &str) {
        let activity = Activity {
            id: id.to_string(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
            kind: ActivityKind::LevelUp { level: 1 },
        };
        self.activities.save(&activity).await.unwrap();
    }

    fn feed_use_case(&self) -> GetFeedUseCase {
        GetFeedUseCase {
            activity_repository: self.activities.clone(),
            friendship_repository: self.friendships.clone(),
            user_repository: self.users.clone(),
        }
    }

    fn profile_use_case(&self) -> GetUserProfileUseCase {
        GetUserProfileUseCase {
            user_repository: self.users.clone(),
            friendship_repository: self.friendships.clone(),
        }
    }
}

fn feed(limit: Option<u16>, cursor: Option<String>) -> GetFeedUseCaseParams {
    GetFeedUseCaseParams {
        user_id: "me".to_string(),
        limit,
        cursor,
    }
}

fn profile(user_id: &str) -> GetUserProfileUseCaseParams {
    GetUserProfileUseCaseParams {
        viewer_id: "me".to_string(),
        user_id: user_id.to_string(),
    }
}

#[test]
fn level_up_only_when_level_increases() {
    assert_eq!(level_up(1, 1), None);
    assert_eq!(level_up(2, 1), None);
    assert_eq!(level_up(1, 3), Some(ActivityKind::LevelUp { level: 3 }));
}

#[tokio::test]
async fn publishes_one_activity_per_kind() {
    let repository = InMemoryActivityRepository::default();
    let kinds = vec![
        ActivityKind::LevelUp { level: 1 },
        ActivityKind::LevelUp { level: 2 },
    ];

    publish_activities(&repository, "u1", kinds).await;

    let activities = repository.all();
    assert_eq!(activities.len(), 2);
    assert!(activities.iter().all(|a| a.user_id == "u1"));
}

#[tokio::test]
async fn leaderboard_ranks_accepted_friends_and_me_with_ties() {
    let context = Context::new();
    let entries = GetLeaderboardUseCase {
        friendship_repository: context.friendships.clone(),
        user_repository: context.users.clone(),
    }
    .execute(GetLeaderboardUseCaseParams {
        user_id: "me".to_string(),
    })
    .await
    .unwrap();

    let rows: Vec<(u32, &str, bool)> = entries
        .iter()
        .map(|e| (e.rank, e.username.as_str(), e.is_me))
        .collect();
    assert_eq!(
        rows,
        [(1, "dave", false), (2, "Bob", false), (2, "carol", true)]
    );
}

#[tokio::test]
async fn feed_merges_friends_activities_newest_first_with_pagination() {
    let context = Context::new();
    for (id, user_id) in [
        ("act1", "f1"),
        ("act2", "f2"),
        ("act3", "f1"),
        ("act4", "f2"),
        ("act5", "f1"),
        ("act6", "me"),
        ("act7", "p1"),
    ] {
        context.add_activity(id, user_id).await;
    }
    let use_case = context.feed_use_case();

    let first = use_case.execute(feed(Some(2), None)).await.unwrap();
    let first_ids: Vec<&str> = first.items.iter().map(|i| i.activity.id.as_str()).collect();
    assert_eq!(first_ids, ["act5", "act4"]);
    assert_eq!(first.items[0].username, "dave");
    assert_eq!(first.next_cursor.as_deref(), Some("act4"));

    let second = use_case
        .execute(feed(Some(2), first.next_cursor.clone()))
        .await
        .unwrap();
    let second_ids: Vec<&str> = second
        .items
        .iter()
        .map(|i| i.activity.id.as_str())
        .collect();
    assert_eq!(second_ids, ["act3", "act2"]);

    let third = use_case
        .execute(feed(Some(2), second.next_cursor.clone()))
        .await
        .unwrap();
    let third_ids: Vec<&str> = third.items.iter().map(|i| i.activity.id.as_str()).collect();
    assert_eq!(third_ids, ["act1"]);
    assert_eq!(third.next_cursor, None);
}

#[tokio::test]
async fn feed_without_friends_is_empty() {
    let context = Context::new();
    context.friendships.delete("me", "f1").await.unwrap();
    context.friendships.delete("me", "f2").await.unwrap();
    context.add_activity("act1", "me").await;

    let page = context
        .feed_use_case()
        .execute(feed(None, None))
        .await
        .unwrap();

    assert!(page.items.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn feed_validates_limit() {
    let context = Context::new();
    let use_case = context.feed_use_case();

    let zero = use_case.execute(feed(Some(0), None)).await;
    let too_big = use_case.execute(feed(Some(51), None)).await;

    assert_eq!(zero.err().unwrap().code, StatusCode::BAD_REQUEST);
    assert_eq!(too_big.err().unwrap().code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn profile_shows_records_only_to_accepted_friends_and_self() {
    let context = Context::new();
    for id in ["me", "f1", "p1"] {
        let mut stored = context.users.get(id).unwrap();
        let record = PersonalRecord {
            weight_kg: 80.0,
            reps: 5,
        };
        stored.personal_records.insert("bench".to_string(), record);
        context.users.insert(stored);
    }
    let use_case = context.profile_use_case();

    let me = use_case.execute(profile("me")).await.unwrap();
    assert!(me.is_me);
    assert!(me.friendship.is_none());
    assert!(me.personal_records.is_some());

    let friend = use_case.execute(profile("f1")).await.unwrap();
    assert_eq!(
        friend.friendship.as_ref().unwrap().status,
        FriendshipStatus::Accepted
    );
    assert!(friend.personal_records.is_some());
    assert_eq!(friend.xp, 300);

    let pending = use_case.execute(profile("p1")).await.unwrap();
    let summary = pending.friendship.as_ref().unwrap();
    assert_eq!(summary.status, FriendshipStatus::Pending);
    assert!(!summary.requested_by_me);
    assert!(pending.personal_records.is_none());
}

#[tokio::test]
async fn profile_of_unknown_user_is_not_found() {
    let context = Context::new();
    let result = context.profile_use_case().execute(profile("ghost")).await;
    assert_eq!(result.err().unwrap().code, StatusCode::NOT_FOUND);
}
