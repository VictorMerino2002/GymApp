use crate::domain::{entities::FriendshipError, value_objects::Error};

pub fn to_error(error: FriendshipError) -> Error {
    let message = error.to_string();
    match error {
        FriendshipError::SelfFriendship => Error::bad_request(&message),
        FriendshipError::NotAddressee => Error::forbidden(&message),
        FriendshipError::AlreadyAccepted => Error::conflict(&message),
    }
}
