use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::FavSessionRepository,
    domain::{
        entities::FavSession,
        value_objects::{Error, Page},
    },
    infrastructure::adapters::repositories::{
        dynamodb_batch::batch_delete_keys,
        dynamodb_pagination::{Item, collect_page},
    },
};

const SESSION_INDEX: &str = "session-index";

pub struct DynamodbFavSessionRepository {
    client: Client,
    table_name: String,
}

impl DynamodbFavSessionRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

fn fav_key(user_id: &str, session_id: &str) -> Item {
    HashMap::from([
        (
            "user_id".to_string(),
            AttributeValue::S(user_id.to_string()),
        ),
        (
            "session_id".to_string(),
            AttributeValue::S(session_id.to_string()),
        ),
    ])
}

fn session_id_cursor(item: &Item) -> Option<String> {
    item.get("session_id")
        .and_then(|id| id.as_s().ok())
        .cloned()
}

#[async_trait]
impl FavSessionRepository for DynamodbFavSessionRepository {
    async fn exists(&self, user_id: &str, session_id: &str) -> Result<bool, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(fav_key(user_id, session_id)))
            .projection_expression("user_id")
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get favorite session"))?;

        Ok(output.item.is_some())
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<FavSession>, Error> {
        let start_key = cursor.map(|session_id| fav_key(user_id, session_id));

        let page = collect_page(limit, start_key, session_id_cursor, |start_key| {
            let request = self
                .client
                .query()
                .table_name(&self.table_name)
                .key_condition_expression("user_id = :user_id")
                .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get favorite sessions"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get favorite sessions"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    async fn save(&self, fav_session: &FavSession) -> Result<(), Error> {
        let item = serde_dynamo::to_item(fav_session)
            .map_err(|_| Error::internal_server("Could not save favorite session"))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save favorite session"))?;

        Ok(())
    }

    async fn delete(&self, user_id: &str, session_id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .set_key(Some(fav_key(user_id, session_id)))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete favorite session"))?;

        Ok(())
    }

    async fn delete_by_session(&self, session_id: &str) -> Result<(), Error> {
        let keys = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name(SESSION_INDEX)
            .key_condition_expression("session_id = :session_id")
            .expression_attribute_values(":session_id", AttributeValue::S(session_id.to_string()))
            .projection_expression("user_id, session_id")
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await
            .map_err(|_| Error::internal_server("Could not delete favorite sessions"))?;

        batch_delete_keys(
            &self.client,
            &self.table_name,
            keys,
            "Could not delete favorite sessions",
        )
        .await
    }
}
