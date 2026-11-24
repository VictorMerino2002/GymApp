use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::RoutineRepository,
    domain::{
        entities::Routine,
        value_objects::{Error, Page},
    },
    infrastructure::adapters::repositories::dynamodb_pagination::{
        Item, collect_page, id_cursor, id_key,
    },
};

const AUTHOR_INDEX: &str = "author-index";
const AUTHOR_CURSOR_SEPARATOR: char = '#';

pub struct DynamodbRoutineRepository {
    client: Client,
    table_name: String,
}

impl DynamodbRoutineRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

fn author_cursor(item: &Item) -> Option<String> {
    let created_at = item.get("created_at")?.as_s().ok()?;
    let id = item.get("id")?.as_s().ok()?;
    Some(format!("{created_at}{AUTHOR_CURSOR_SEPARATOR}{id}"))
}

fn author_start_key(author_id: &str, cursor: &str) -> Result<Item, Error> {
    let (created_at, id) = cursor
        .split_once(AUTHOR_CURSOR_SEPARATOR)
        .ok_or_else(|| Error::bad_request("Invalid cursor"))?;

    Ok(HashMap::from([
        ("id".to_string(), AttributeValue::S(id.to_string())),
        (
            "author_id".to_string(),
            AttributeValue::S(author_id.to_string()),
        ),
        (
            "created_at".to_string(),
            AttributeValue::S(created_at.to_string()),
        ),
    ]))
}

#[async_trait]
impl RoutineRepository for DynamodbRoutineRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Routine>, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get routine"))?;

        output
            .item
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get routine"))
            })
            .transpose()
    }

    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Routine>, Error> {
        let page = collect_page(limit, cursor.map(id_key), id_cursor, |start_key| {
            let request = self
                .client
                .scan()
                .table_name(&self.table_name)
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get routines"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get routines"))?;

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
    ) -> Result<Page<Routine>, Error> {
        let start_key = cursor
            .map(|cursor| author_start_key(author_id, cursor))
            .transpose()?;

        let page = collect_page(limit, start_key, author_cursor, |start_key| {
            let request = self
                .client
                .query()
                .table_name(&self.table_name)
                .index_name(AUTHOR_INDEX)
                .key_condition_expression("author_id = :author_id")
                .expression_attribute_values(":author_id", AttributeValue::S(author_id.to_string()))
                .scan_index_forward(false)
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get routines"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get routines"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    async fn save(&self, routine: &Routine) -> Result<(), Error> {
        let mut item: Item = serde_dynamo::to_item(routine)
            .map_err(|_| Error::internal_server("Could not save routine"))?;

        item.insert(
            "author_id".to_string(),
            AttributeValue::S(routine.author.user_id.clone()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save routine"))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete routine"))?;

        Ok(())
    }
}
