use crate::{
    application::ports::repositories::FriendshipRepository,
    domain::{entities::FriendshipStatus, value_objects::Error},
};

const PAGE_SIZE: u16 = 100;
pub const MAX_FRIENDS: usize = 500;

pub async fn accepted_friend_ids(
    friendship_repository: &dyn FriendshipRepository,
    user_id: &str,
) -> Result<Vec<String>, Error> {
    let mut friend_ids = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let page = friendship_repository
            .find_by_user(
                user_id,
                Some(FriendshipStatus::Accepted),
                PAGE_SIZE,
                cursor.as_deref(),
            )
            .await?;

        friend_ids.extend(page.items.iter().map(|friendship| {
            if friendship.user1_id() == user_id {
                friendship.user2_id().to_string()
            } else {
                friendship.user1_id().to_string()
            }
        }));

        match page.next_cursor {
            Some(next) if friend_ids.len() < MAX_FRIENDS => cursor = Some(next),
            _ => break,
        }
    }

    friend_ids.truncate(MAX_FRIENDS);
    Ok(friend_ids)
}
