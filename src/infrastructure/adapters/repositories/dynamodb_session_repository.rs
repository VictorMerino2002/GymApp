use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::SessionRepository,
    domain::{
        entities::Session,
        value_objects::{Error, Page},
    },
};

const AUTHOR_INDEX: &str = "author-index";

pub struct DynamodbSessionRepository {
    client: Client,
    table_name: String,
}

impl DynamodbSessionRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl SessionRepository for DynamodbSessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get session"))?;

        output
            .item
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get session"))
            })
            .transpose()
    }

    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Session>, Error> {
        let start_key =
            cursor.map(|id| HashMap::from([("id".to_string(), AttributeValue::S(id.to_string()))]));

        let output = self
            .client
            .scan()
            .table_name(&self.table_name)
            .limit(i32::from(limit))
            .set_exclusive_start_key(start_key)
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get sessions"))?;

        let items = serde_dynamo::from_items(output.items.unwrap_or_default())
            .map_err(|_| Error::internal_server("Could not get sessions"))?;

        let next_cursor = output
            .last_evaluated_key
            .and_then(|key| key.get("id").and_then(|id| id.as_s().ok()).cloned());

        Ok(Page { items, next_cursor })
    }

    async fn find_by_author(
        &self,
        author_id: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Session>, Error> {
        let start_key = cursor.map(|id| {
            HashMap::from([
                ("id".to_string(), AttributeValue::S(id.to_string())),
                (
                    "author_id".to_string(),
                    AttributeValue::S(author_id.to_string()),
                ),
            ])
        });

        let output = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name(AUTHOR_INDEX)
            .key_condition_expression("author_id = :author_id")
            .expression_attribute_values(":author_id", AttributeValue::S(author_id.to_string()))
            .limit(i32::from(limit))
            .set_exclusive_start_key(start_key)
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get sessions"))?;

        let items = serde_dynamo::from_items(output.items.unwrap_or_default())
            .map_err(|_| Error::internal_server("Could not get sessions"))?;

        let next_cursor = output
            .last_evaluated_key
            .and_then(|key| key.get("id").and_then(|id| id.as_s().ok()).cloned());

        Ok(Page { items, next_cursor })
    }

    async fn save(&self, session: &Session) -> Result<(), Error> {
        let mut item: HashMap<String, AttributeValue> = serde_dynamo::to_item(session)
            .map_err(|_| Error::internal_server("Could not save session"))?;

        item.insert(
            "author_id".to_string(),
            AttributeValue::S(session.author.user_id.clone()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save session"))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete session"))?;

        Ok(())
    }
}
