use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::FriendshipRepository,
    domain::{
        entities::{Friendship, FriendshipStatus},
        value_objects::{Error, Page},
    },
    infrastructure::adapters::repositories::{
        dynamodb_batch::delete_partition,
        dynamodb_pagination::{Item, RawPage, collect_page},
    },
};

const USER2_INDEX: &str = "user2-index";
const CURSOR_SEPARATOR: char = '#';

pub struct DynamodbFriendshipRepository {
    client: Client,
    table_name: String,
}

impl DynamodbFriendshipRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    User1,
    User2,
}

impl Side {
    fn tag(self) -> &'static str {
        match self {
            Self::User1 => "1",
            Self::User2 => "2",
        }
    }

    fn own_attribute(self) -> &'static str {
        match self {
            Self::User1 => "user1_id",
            Self::User2 => "user2_id",
        }
    }

    fn other_attribute(self) -> &'static str {
        match self {
            Self::User1 => "user2_id",
            Self::User2 => "user1_id",
        }
    }

    fn index(self) -> Option<&'static str> {
        match self {
            Self::User1 => None,
            Self::User2 => Some(USER2_INDEX),
        }
    }

    fn start_key(self, user_id: &str, other_id: &str) -> Item {
        HashMap::from([
            (
                self.own_attribute().to_string(),
                AttributeValue::S(user_id.to_string()),
            ),
            (
                self.other_attribute().to_string(),
                AttributeValue::S(other_id.to_string()),
            ),
        ])
    }

    fn cursor(self, item: &Item) -> Option<String> {
        let other_id = item.get(self.other_attribute())?.as_s().ok()?;
        Some(format!("{}{CURSOR_SEPARATOR}{other_id}", self.tag()))
    }

    fn start_cursor(self) -> String {
        format!("{}{CURSOR_SEPARATOR}", self.tag())
    }
}

fn parse_cursor(cursor: Option<&str>) -> Result<(Side, Option<&str>), Error> {
    let Some(cursor) = cursor else {
        return Ok((Side::User1, None));
    };

    let (tag, other_id) = cursor
        .split_once(CURSOR_SEPARATOR)
        .ok_or_else(|| Error::bad_request("Invalid cursor"))?;

    let side = match tag {
        "1" => Side::User1,
        "2" => Side::User2,
        _ => return Err(Error::bad_request("Invalid cursor")),
    };

    Ok((side, Some(other_id).filter(|id| !id.is_empty())))
}

fn pair_key(user_a: &str, user_b: &str) -> Item {
    let (user1_id, user2_id) = Friendship::canonical_pair(user_a.to_string(), user_b.to_string());
    HashMap::from([
        ("user1_id".to_string(), AttributeValue::S(user1_id)),
        ("user2_id".to_string(), AttributeValue::S(user2_id)),
    ])
}

impl DynamodbFriendshipRepository {
    async fn collect_side(
        &self,
        side: Side,
        user_id: &str,
        status: Option<&AttributeValue>,
        limit: u16,
        other_id: Option<&str>,
    ) -> Result<RawPage, Error> {
        let start_key = other_id.map(|other_id| side.start_key(user_id, other_id));
        let key_condition = format!("{} = :user_id", side.own_attribute());

        collect_page(
            limit,
            start_key,
            |item| side.cursor(item),
            |start_key| {
                let mut request = self
                    .client
                    .query()
                    .table_name(&self.table_name)
                    .set_index_name(side.index().map(str::to_string))
                    .key_condition_expression(&key_condition)
                    .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
                    .limit(i32::from(limit))
                    .set_exclusive_start_key(start_key);

                if let Some(status) = status {
                    request = request
                        .filter_expression("#status = :status")
                        .expression_attribute_names("#status", "status")
                        .expression_attribute_values(":status", status.clone());
                }

                async move {
                    let output = request
                        .send()
                        .await
                        .map_err(|_| Error::internal_server("Could not get friendships"))?;
                    Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
                }
            },
        )
        .await
    }
}

#[async_trait]
impl FriendshipRepository for DynamodbFriendshipRepository {
    async fn find(&self, user_a: &str, user_b: &str) -> Result<Option<Friendship>, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(pair_key(user_a, user_b)))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get friendship"))?;

        output
            .item
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get friendship"))
            })
            .transpose()
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        status: Option<FriendshipStatus>,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Friendship>, Error> {
        let status = status
            .map(serde_dynamo::to_attribute_value)
            .transpose()
            .map_err(|_| Error::internal_server("Could not build status filter"))?;
        let (side, other_id) = parse_cursor(cursor)?;

        let (items, next_cursor) = match side {
            Side::User2 => {
                let page = self
                    .collect_side(Side::User2, user_id, status.as_ref(), limit, other_id)
                    .await?;
                (page.items, page.next_cursor)
            }
            Side::User1 => {
                let mut page = self
                    .collect_side(Side::User1, user_id, status.as_ref(), limit, other_id)
                    .await?;

                let remaining = limit.saturating_sub(page.items.len() as u16);

                if page.next_cursor.is_some() {
                    (page.items, page.next_cursor)
                } else if remaining == 0 {
                    (page.items, Some(Side::User2.start_cursor()))
                } else {
                    let rest = self
                        .collect_side(Side::User2, user_id, status.as_ref(), remaining, None)
                        .await?;
                    page.items.extend(rest.items);
                    (page.items, rest.next_cursor)
                }
            }
        };

        let items = serde_dynamo::from_items(items)
            .map_err(|_| Error::internal_server("Could not get friendships"))?;

        Ok(Page { items, next_cursor })
    }

    async fn save(&self, friendship: &Friendship) -> Result<(), Error> {
        let item = serde_dynamo::to_item(friendship)
            .map_err(|_| Error::internal_server("Could not save friendship"))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save friendship"))?;

        Ok(())
    }

    async fn delete(&self, user_a: &str, user_b: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .set_key(Some(pair_key(user_a, user_b)))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete friendship"))?;

        Ok(())
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        for side in [Side::User1, Side::User2] {
            delete_partition(
                &self.client,
                &self.table_name,
                side.index(),
                side.own_attribute(),
                user_id,
                ["user1_id", "user2_id"],
                "Could not delete friendships",
            )
            .await?;
        }

        Ok(())
    }
}
