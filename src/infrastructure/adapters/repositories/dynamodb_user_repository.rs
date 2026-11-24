use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{
    Client,
    operation::transact_write_items::TransactWriteItemsError,
    types::{AttributeValue, Put, TransactWriteItem},
};

use crate::{
    application::ports::repositories::UserRepository,
    domain::{
        entities::User,
        value_objects::{Error, Page},
    },
    infrastructure::adapters::repositories::{
        dynamodb_batch::batch_get_by_ids,
        dynamodb_pagination::{Item, collect_page},
    },
};

const USERNAME_SEARCH_INDEX: &str = "username-search-index";
const SEARCH_PARTITION: &str = "user";
const CURSOR_SEPARATOR: char = '#';
const USERNAME_MARKER_PREFIX: &str = "username#";

pub struct DynamodbUserRepository {
    client: Client,
    table_name: String,
}

impl DynamodbUserRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

fn user_item(user: &User) -> Result<Item, Error> {
    let mut item: Item =
        serde_dynamo::to_item(user).map_err(|_| Error::internal_server("Could not save user"))?;

    item.insert(
        "search_pk".to_string(),
        AttributeValue::S(SEARCH_PARTITION.to_string()),
    );
    item.insert(
        "username_lower".to_string(),
        AttributeValue::S(user.username.to_lowercase()),
    );

    Ok(item)
}

fn username_marker_item(user: &User) -> Item {
    HashMap::from([
        (
            "id".to_string(),
            AttributeValue::S(format!(
                "{USERNAME_MARKER_PREFIX}{}",
                user.username.to_lowercase()
            )),
        ),
        ("user_id".to_string(), AttributeValue::S(user.id.clone())),
    ])
}

fn new_item_put(table_name: &str, item: Item) -> Result<TransactWriteItem, Error> {
    let put = Put::builder()
        .table_name(table_name)
        .set_item(Some(item))
        .condition_expression("attribute_not_exists(id)")
        .build()
        .map_err(|_| Error::internal_server("Could not create user"))?;

    Ok(TransactWriteItem::builder().put(put).build())
}

fn search_cursor(item: &Item) -> Option<String> {
    let username_lower = item.get("username_lower")?.as_s().ok()?;
    let id = item.get("id")?.as_s().ok()?;
    Some(format!("{username_lower}{CURSOR_SEPARATOR}{id}"))
}

fn search_start_key(cursor: &str) -> Result<Item, Error> {
    let (username_lower, id) = cursor
        .rsplit_once(CURSOR_SEPARATOR)
        .ok_or_else(|| Error::bad_request("Invalid cursor"))?;

    Ok(HashMap::from([
        ("id".to_string(), AttributeValue::S(id.to_string())),
        (
            "search_pk".to_string(),
            AttributeValue::S(SEARCH_PARTITION.to_string()),
        ),
        (
            "username_lower".to_string(),
            AttributeValue::S(username_lower.to_string()),
        ),
    ]))
}

#[async_trait]
impl UserRepository for DynamodbUserRepository {
    async fn create(&self, user: &User) -> Result<(), Error> {
        let result = self
            .client
            .transact_write_items()
            .transact_items(new_item_put(&self.table_name, username_marker_item(user))?)
            .transact_items(new_item_put(&self.table_name, user_item(user)?)?)
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                let username_taken = matches!(
                    error.as_service_error(),
                    Some(TransactWriteItemsError::TransactionCanceledException(canceled))
                        if canceled
                            .cancellation_reasons()
                            .iter()
                            .any(|reason| reason.code() == Some("ConditionalCheckFailed"))
                );

                if username_taken {
                    Err(Error::conflict("Username already taken"))
                } else {
                    Err(Error::internal_server("Could not create user"))
                }
            }
        }
    }

    async fn update(&self, user: &User) -> Result<(), Error> {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(user_item(user)?))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save user"))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        if id.starts_with(USERNAME_MARKER_PREFIX) {
            return Ok(None);
        }

        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get user"))?;

        output
            .item
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get user"))
            })
            .transpose()
    }

    async fn update_progress(
        &self,
        user: &User,
        expected_exercises_done: u32,
    ) -> Result<bool, Error> {
        let stats = serde_dynamo::to_attribute_value(user.stats)
            .map_err(|_| Error::internal_server("Could not update user stats"))?;
        let records = serde_dynamo::to_attribute_value(&user.personal_records)
            .map_err(|_| Error::internal_server("Could not update user stats"))?;

        let result = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(user.id.clone()))
            .update_expression("SET #stats = :stats, #records = :records")
            .condition_expression("attribute_exists(id) AND #stats.#done = :expected")
            .expression_attribute_names("#stats", "stats")
            .expression_attribute_names("#records", "personal_records")
            .expression_attribute_names("#done", "exercises_done")
            .expression_attribute_values(":stats", stats)
            .expression_attribute_values(":records", records)
            .expression_attribute_values(
                ":expected",
                AttributeValue::N(expected_exercises_done.to_string()),
            )
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(error)
                if error
                    .as_service_error()
                    .is_some_and(|error| error.is_conditional_check_failed_exception()) =>
            {
                Ok(false)
            }
            Err(_) => Err(Error::internal_server("Could not update user stats")),
        }
    }

    async fn increment_token_version(&self, user_id: &str) -> Result<(), Error> {
        let result = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(user_id.to_string()))
            .update_expression("ADD #version :one")
            .condition_expression("attribute_exists(id)")
            .expression_attribute_names("#version", "token_version")
            .expression_attribute_values(":one", AttributeValue::N("1".to_string()))
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(error)
                if error
                    .as_service_error()
                    .is_some_and(|error| error.is_conditional_check_failed_exception()) =>
            {
                Err(Error::unauthorized("User not found"))
            }
            Err(_) => Err(Error::internal_server("Could not log out")),
        }
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<User>, Error> {
        let items =
            batch_get_by_ids(&self.client, &self.table_name, ids, "Could not get users").await?;

        serde_dynamo::from_items(items).map_err(|_| Error::internal_server("Could not get users"))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let output = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name(USERNAME_SEARCH_INDEX)
            .key_condition_expression("search_pk = :search_pk AND username_lower = :username")
            .expression_attribute_values(
                ":search_pk",
                AttributeValue::S(SEARCH_PARTITION.to_string()),
            )
            .expression_attribute_values(":username", AttributeValue::S(username.to_lowercase()))
            .limit(1)
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get user"))?;

        output
            .items
            .and_then(|items| items.into_iter().next())
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get user"))
            })
            .transpose()
    }

    async fn search_by_username(
        &self,
        prefix: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<User>, Error> {
        let start_key = cursor.map(search_start_key).transpose()?;
        let prefix = prefix.to_lowercase();

        let page = collect_page(limit, start_key, search_cursor, |start_key| {
            let request = self
                .client
                .query()
                .table_name(&self.table_name)
                .index_name(USERNAME_SEARCH_INDEX)
                .key_condition_expression(
                    "search_pk = :search_pk AND begins_with(username_lower, :prefix)",
                )
                .expression_attribute_values(
                    ":search_pk",
                    AttributeValue::S(SEARCH_PARTITION.to_string()),
                )
                .expression_attribute_values(":prefix", AttributeValue::S(prefix.clone()))
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get users"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get users"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }
}
