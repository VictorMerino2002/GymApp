use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{
    Client,
    operation::transact_write_items::TransactWriteItemsError,
    types::{AttributeValue, Delete, Put, ReturnValue, TransactWriteItem, Update},
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

fn username_marker_key(username: &str) -> Item {
    HashMap::from([(
        "id".to_string(),
        AttributeValue::S(format!(
            "{USERNAME_MARKER_PREFIX}{}",
            username.to_lowercase()
        )),
    )])
}

fn username_marker_item(user: &User) -> Item {
    let mut item = username_marker_key(&user.username);
    item.insert("user_id".to_string(), AttributeValue::S(user.id.clone()));
    item
}

fn profile_update(table_name: &str, user: &User) -> Result<Update, Error> {
    let picture_url = match &user.picture_url {
        Some(url) => AttributeValue::S(url.clone()),
        None => AttributeValue::Null(true),
    };

    Update::builder()
        .table_name(table_name)
        .key("id", AttributeValue::S(user.id.clone()))
        .update_expression(
            "SET #username = :username, #username_lower = :username_lower, #picture = :picture",
        )
        .condition_expression("attribute_exists(id)")
        .expression_attribute_names("#username", "username")
        .expression_attribute_names("#username_lower", "username_lower")
        .expression_attribute_names("#picture", "picture_url")
        .expression_attribute_values(":username", AttributeValue::S(user.username.clone()))
        .expression_attribute_values(
            ":username_lower",
            AttributeValue::S(user.username.to_lowercase()),
        )
        .expression_attribute_values(":picture", picture_url)
        .build()
        .map_err(|_| Error::internal_server("Could not update user"))
}

fn is_condition_failure(error: &TransactWriteItemsError) -> bool {
    matches!(
        error,
        TransactWriteItemsError::TransactionCanceledException(canceled)
            if canceled
                .cancellation_reasons()
                .iter()
                .any(|reason| reason.code() == Some("ConditionalCheckFailed"))
    )
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
                let username_taken = error.as_service_error().is_some_and(is_condition_failure);

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

    async fn update_profile(&self, user: &User, previous_username: &str) -> Result<(), Error> {
        let username_changed = user.username.to_lowercase() != previous_username.to_lowercase();
        let mut request = self.client.transact_write_items();

        if username_changed {
            let delete_old_marker = Delete::builder()
                .table_name(&self.table_name)
                .set_key(Some(username_marker_key(previous_username)))
                .build()
                .map_err(|_| Error::internal_server("Could not update user"))?;

            request = request
                .transact_items(
                    TransactWriteItem::builder()
                        .delete(delete_old_marker)
                        .build(),
                )
                .transact_items(new_item_put(&self.table_name, username_marker_item(user))?);
        }

        let update = profile_update(&self.table_name, user)?;
        let result = request
            .transact_items(TransactWriteItem::builder().update(update).build())
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) if error.as_service_error().is_some_and(is_condition_failure) => {
                if username_changed {
                    Err(Error::conflict("Username already taken"))
                } else {
                    Err(Error::not_found("User not found"))
                }
            }
            Err(_) => Err(Error::internal_server("Could not update user")),
        }
    }

    async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<u32, Error> {
        let result = self
            .client
            .update_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(user_id.to_string()))
            .update_expression("SET #password = :password ADD #version :one")
            .condition_expression("attribute_exists(id)")
            .expression_attribute_names("#password", "password_hash")
            .expression_attribute_names("#version", "token_version")
            .expression_attribute_values(":password", AttributeValue::S(password_hash.to_string()))
            .expression_attribute_values(":one", AttributeValue::N("1".to_string()))
            .return_values(ReturnValue::UpdatedNew)
            .send()
            .await;

        let output = match result {
            Ok(output) => output,
            Err(error)
                if error
                    .as_service_error()
                    .is_some_and(|error| error.is_conditional_check_failed_exception()) =>
            {
                return Err(Error::unauthorized("User not found"));
            }
            Err(_) => return Err(Error::internal_server("Could not update password")),
        };

        output
            .attributes()
            .and_then(|attributes| attributes.get("token_version"))
            .and_then(|version| version.as_n().ok())
            .and_then(|version| version.parse().ok())
            .ok_or_else(|| Error::internal_server("Could not update password"))
    }

    async fn delete(&self, user: &User) -> Result<(), Error> {
        let delete_marker = Delete::builder()
            .table_name(&self.table_name)
            .set_key(Some(username_marker_key(&user.username)))
            .build()
            .map_err(|_| Error::internal_server("Could not delete user"))?;

        let delete_user = Delete::builder()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(user.id.clone()))
            .build()
            .map_err(|_| Error::internal_server("Could not delete user"))?;

        self.client
            .transact_write_items()
            .transact_items(TransactWriteItem::builder().delete(delete_marker).build())
            .transact_items(TransactWriteItem::builder().delete(delete_user).build())
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete user"))?;

        Ok(())
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
