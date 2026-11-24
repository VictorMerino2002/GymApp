use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

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

const USERNAME_INDEX: &str = "username-index";
const USERNAME_SEARCH_INDEX: &str = "username-search-index";
const SEARCH_PARTITION: &str = "user";
const CURSOR_SEPARATOR: char = '#';

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
    async fn save(&self, user: &User) -> Result<(), Error> {
        let mut item: Item = serde_dynamo::to_item(user)
            .map_err(|_| Error::internal_server("Could not save user"))?;

        item.insert(
            "search_pk".to_string(),
            AttributeValue::S(SEARCH_PARTITION.to_string()),
        );
        item.insert(
            "username_lower".to_string(),
            AttributeValue::S(user.username.to_lowercase()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save user"))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, Error> {
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
            .index_name(USERNAME_INDEX)
            .key_condition_expression("username = :username")
            .expression_attribute_values(":username", AttributeValue::S(username.to_string()))
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
