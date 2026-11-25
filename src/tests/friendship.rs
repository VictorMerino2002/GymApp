use std::sync::Arc;

use axum::http::StatusCode;

use crate::{
    application::{
        ports::repositories::FriendshipRepository,
        use_cases::friendships::{
            AcceptFriendRequestUseCase, AcceptFriendRequestUseCaseParams, ListFriendsUseCase,
            ListFriendsUseCaseParams, RemoveFriendshipUseCase, RemoveFriendshipUseCaseParams,
            SendFriendRequestUseCase, SendFriendRequestUseCaseParams,
        },
    },
    domain::{
        entities::{Friendship, FriendshipError, FriendshipStatus},
        value_objects::Error,
    },
    tests::support::{InMemoryFriendshipRepository, InMemoryUserRepository, user},
};

#[test]
fn pair_is_canonical_regardless_of_order() {
    let a = Friendship::request("b".into(), "a".into()).unwrap();
    let b = Friendship::request("a".into(), "b".into()).unwrap();
    assert_eq!(a.user1_id(), b.user1_id());
    assert_eq!(a.user2_id(), b.user2_id());
    assert_eq!(a.user1_id(), "a");
}

#[test]
fn rejects_self_friendship() {
    assert_eq!(
        Friendship::request("a".into(), "a".into()),
        Err(FriendshipError::SelfFriendship)
    );
}

#[test]
fn only_addressee_can_accept() {
    let mut f = Friendship::request("a".into(), "b".into()).unwrap();
    assert_eq!(f.accept("a"), Err(FriendshipError::NotAddressee));
    assert_eq!(f.accept("c"), Err(FriendshipError::NotAddressee));
    assert_eq!(f.accept("b"), Ok(()));
    assert_eq!(f.status(), FriendshipStatus::Accepted);
    assert_eq!(f.accept("b"), Err(FriendshipError::AlreadyAccepted));
}

struct Context {
    users: Arc<InMemoryUserRepository>,
    friendships: Arc<InMemoryFriendshipRepository>,
}

impl Context {
    fn new() -> Self {
        Self {
            users: Arc::new(InMemoryUserRepository::with(vec![
                user("u1", "alice"),
                user("u2", "bob"),
                user("u3", "carol"),
            ])),
            friendships: Arc::default(),
        }
    }

    async fn send(&self, from: &str, to: &str) -> Result<Friendship, Error> {
        SendFriendRequestUseCase {
            friendship_repository: self.friendships.clone(),
            user_repository: self.users.clone(),
        }
        .execute(SendFriendRequestUseCaseParams {
            user_id: from.to_string(),
            friend_id: to.to_string(),
        })
        .await
    }

    async fn accept(&self, by: &str, other: &str) -> Result<Friendship, Error> {
        AcceptFriendRequestUseCase {
            friendship_repository: self.friendships.clone(),
        }
        .execute(AcceptFriendRequestUseCaseParams {
            user_id: by.to_string(),
            friend_id: other.to_string(),
        })
        .await
    }

    async fn remove(&self, by: &str, other: &str) -> Result<(), Error> {
        RemoveFriendshipUseCase {
            friendship_repository: self.friendships.clone(),
        }
        .execute(RemoveFriendshipUseCaseParams {
            user_id: by.to_string(),
            friend_id: other.to_string(),
        })
        .await
    }

    async fn list(
        &self,
        status: Option<FriendshipStatus>,
        limit: Option<u16>,
    ) -> Result<Vec<(String, FriendshipStatus, bool)>, Error> {
        let page = ListFriendsUseCase {
            friendship_repository: self.friendships.clone(),
            user_repository: self.users.clone(),
        }
        .execute(ListFriendsUseCaseParams {
            user_id: "u1".to_string(),
            status,
            limit,
            cursor: None,
        })
        .await?;

        let mut friends: Vec<(String, FriendshipStatus, bool)> = page
            .items
            .into_iter()
            .map(|f| (f.username, f.status, f.requested_by_me))
            .collect();
        friends.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(friends)
    }
}

#[tokio::test]
async fn send_creates_pending_request() {
    let context = Context::new();
    let friendship = context.send("u2", "u1").await.unwrap();

    assert_eq!(friendship.status(), FriendshipStatus::Pending);
    assert_eq!(friendship.requested_by(), "u2");
    assert_eq!(
        context.friendships.find("u1", "u2").await.unwrap(),
        Some(friendship)
    );
}

#[tokio::test]
async fn send_rejects_invalid_targets() {
    let context = Context::new();
    let to_self = context.send("u1", "u1").await.unwrap_err();
    let to_ghost = context.send("u1", "ghost").await.unwrap_err();

    assert_eq!(to_self.code, StatusCode::BAD_REQUEST);
    assert_eq!(to_ghost.code, StatusCode::NOT_FOUND);
    assert_eq!(context.friendships.count(), 0);
}

#[tokio::test]
async fn sending_twice_is_a_conflict() {
    let context = Context::new();
    context.send("u1", "u2").await.unwrap();

    let error = context.send("u1", "u2").await.unwrap_err();
    assert_eq!(error.code, StatusCode::CONFLICT);
}

#[tokio::test]
async fn mutual_request_accepts_the_friendship() {
    let context = Context::new();
    context.send("u1", "u2").await.unwrap();
    let friendship = context.send("u2", "u1").await.unwrap();

    assert_eq!(friendship.status(), FriendshipStatus::Accepted);
    assert_eq!(context.friendships.count(), 1);

    let error = context.send("u1", "u2").await.unwrap_err();
    assert_eq!(error.code, StatusCode::CONFLICT);
}

#[tokio::test]
async fn accept_rules() {
    let context = Context::new();
    context.send("u1", "u2").await.unwrap();

    let by_requester = context.accept("u1", "u2").await.unwrap_err();
    let by_addressee = context.accept("u2", "u1").await.unwrap();
    let again = context.accept("u2", "u1").await.unwrap_err();

    assert_eq!(by_requester.code, StatusCode::FORBIDDEN);
    assert_eq!(by_addressee.status(), FriendshipStatus::Accepted);
    assert_eq!(again.code, StatusCode::CONFLICT);
}

#[tokio::test]
async fn accepting_missing_request_is_not_found() {
    let context = Context::new();
    let error = context.accept("u2", "u1").await.unwrap_err();
    assert_eq!(error.code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn either_side_can_remove() {
    let context = Context::new();
    context.send("u1", "u2").await.unwrap();
    context.remove("u2", "u1").await.unwrap();

    assert_eq!(context.friendships.count(), 0);
    let error = context.remove("u1", "u2").await.unwrap_err();
    assert_eq!(error.code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_shows_friend_side_and_filters_by_status() {
    let context = Context::new();
    context.send("u1", "u2").await.unwrap();
    context.send("u3", "u1").await.unwrap();
    context.accept("u1", "u3").await.unwrap();

    assert_eq!(
        context.list(None, None).await.unwrap(),
        [
            ("bob".to_string(), FriendshipStatus::Pending, true),
            ("carol".to_string(), FriendshipStatus::Accepted, false),
        ]
    );
    assert_eq!(
        context
            .list(Some(FriendshipStatus::Accepted), None)
            .await
            .unwrap(),
        [("carol".to_string(), FriendshipStatus::Accepted, false)]
    );
}

#[tokio::test]
async fn list_validates_limit() {
    let context = Context::new();
    let zero = context.list(None, Some(0)).await.unwrap_err();
    let too_big = context.list(None, Some(101)).await.unwrap_err();

    assert_eq!(zero.code, StatusCode::BAD_REQUEST);
    assert_eq!(too_big.code, StatusCode::BAD_REQUEST);
}
