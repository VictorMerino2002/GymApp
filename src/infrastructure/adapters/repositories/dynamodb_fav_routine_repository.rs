use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::FavRoutineRepository,
    domain::{
        entities::FavRoutine,
        value_objects::{Error, Page},
    },
    infrastructure::adapters::repositories::dynamodb_pagination::{Item, collect_page},
};

pub struct DynamodbFavRoutineRepository {
    client: Client,
    table_name: String,
}

impl DynamodbFavRoutineRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

fn fav_key(user_id: &str, routine_id: &str) -> Item {
    HashMap::from([
        (
            "user_id".to_string(),
            AttributeValue::S(user_id.to_string()),
        ),
        (
            "routine_id".to_string(),
            AttributeValue::S(routine_id.to_string()),
        ),
    ])
}

fn routine_id_cursor(item: &Item) -> Option<String> {
    item.get("routine_id")
        .and_then(|id| id.as_s().ok())
        .cloned()
}

#[async_trait]
impl FavRoutineRepository for DynamodbFavRoutineRepository {
    async fn exists(&self, user_id: &str, routine_id: &str) -> Result<bool, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(fav_key(user_id, routine_id)))
            .projection_expression("user_id")
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get favorite routine"))?;

        Ok(output.item.is_some())
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<FavRoutine>, Error> {
        let start_key = cursor.map(|routine_id| fav_key(user_id, routine_id));

        let page = collect_page(limit, start_key, routine_id_cursor, |start_key| {
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
                    .map_err(|_| Error::internal_server("Could not get favorite routines"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get favorite routines"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    async fn save(&self, fav_routine: &FavRoutine) -> Result<(), Error> {
        let item = serde_dynamo::to_item(fav_routine)
            .map_err(|_| Error::internal_server("Could not save favorite routine"))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save favorite routine"))?;

        Ok(())
    }

    async fn delete(&self, user_id: &str, routine_id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .set_key(Some(fav_key(user_id, routine_id)))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete favorite routine"))?;

        Ok(())
    }
}
