use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FriendshipStatus {
    Pending,
    Accepted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendshipError {
    SelfFriendship,
    NotAddressee,
    AlreadyAccepted,
}

impl fmt::Display for FriendshipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfFriendship => write!(f, "a user cannot befriend themselves"),
            Self::NotAddressee => write!(f, "only the addressee can accept the request"),
            Self::AlreadyAccepted => write!(f, "friendship already accepted"),
        }
    }
}

impl std::error::Error for FriendshipError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Friendship {
    user1_id: String,
    user2_id: String,
    requested_by: String,
    status: FriendshipStatus,
}

impl Friendship {
    pub fn request(requester_id: String, addressee_id: String) -> Result<Self, FriendshipError> {
        if requester_id == addressee_id {
            return Err(FriendshipError::SelfFriendship);
        }
        let (user1_id, user2_id) = Self::canonical_pair(requester_id.clone(), addressee_id);
        Ok(Self {
            user1_id,
            user2_id,
            requested_by: requester_id,
            status: FriendshipStatus::Pending,
        })
    }

    pub fn accept(&mut self, user_id: &str) -> Result<(), FriendshipError> {
        if self.status == FriendshipStatus::Accepted {
            return Err(FriendshipError::AlreadyAccepted);
        }
        if user_id == self.requested_by || !self.involves(user_id) {
            return Err(FriendshipError::NotAddressee);
        }
        self.status = FriendshipStatus::Accepted;
        Ok(())
    }

    pub fn canonical_pair(a: String, b: String) -> (String, String) {
        if a <= b { (a, b) } else { (b, a) }
    }

    pub fn involves(&self, user_id: &str) -> bool {
        self.user1_id == user_id || self.user2_id == user_id
    }

    pub fn user1_id(&self) -> &str {
        &self.user1_id
    }

    pub fn user2_id(&self) -> &str {
        &self.user2_id
    }

    pub fn requested_by(&self) -> &str {
        &self.requested_by
    }

    pub fn status(&self) -> FriendshipStatus {
        self.status
    }
}
