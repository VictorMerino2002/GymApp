use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::SessionRepository,
    domain::{
        entities::Session,
        value_objects::{Error, Muscle, Page},
    },
    infrastructure::adapters::repositories::{
        dynamodb_batch::batch_get_by_ids,
        dynamodb_pagination::{collect_page, id_cursor, id_key, muscle_value},
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

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<Session>, Error> {
        let items = batch_get_by_ids(
            &self.client,
            &self.table_name,
            ids,
            "Could not get sessions",
        )
        .await?;

        serde_dynamo::from_items(items)
            .map_err(|_| Error::internal_server("Could not get sessions"))
    }

    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
        muscle: Option<Muscle>,
    ) -> Result<Page<Session>, Error> {
        let muscle = muscle.map(muscle_value).transpose()?;

        let page = collect_page(limit, cursor.map(id_key), id_cursor, |start_key| {
            let mut request = self
                .client
                .scan()
                .table_name(&self.table_name)
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            if let Some(muscle) = &muscle {
                request = request
                    .filter_expression("contains(#muscles, :muscle)")
                    .expression_attribute_names("#muscles", "muscles")
                    .expression_attribute_values(":muscle", muscle.clone());
            }

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get sessions"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get sessions"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    async fn find_by_author(
        &self,
        author_id: &str,
        limit: u16,
        cursor: Option<&str>,
        muscle: Option<Muscle>,
    ) -> Result<Page<Session>, Error> {
        let muscle = muscle.map(muscle_value).transpose()?;
        let start_key = cursor.map(|id| {
            let mut key = id_key(id);
            key.insert(
                "author_id".to_string(),
                AttributeValue::S(author_id.to_string()),
            );
            key
        });

        let page = collect_page(limit, start_key, id_cursor, |start_key| {
            let mut request = self
                .client
                .query()
                .table_name(&self.table_name)
                .index_name(AUTHOR_INDEX)
                .key_condition_expression("author_id = :author_id")
                .expression_attribute_values(":author_id", AttributeValue::S(author_id.to_string()))
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            if let Some(muscle) = &muscle {
                request = request
                    .filter_expression("contains(#muscles, :muscle)")
                    .expression_attribute_names("#muscles", "muscles")
                    .expression_attribute_values(":muscle", muscle.clone());
            }

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get sessions"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get sessions"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
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
