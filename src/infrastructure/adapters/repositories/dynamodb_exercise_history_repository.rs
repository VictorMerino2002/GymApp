use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::ExerciseHistoryRepository,
    domain::{
        entities::ExerciseHistory,
        value_objects::{Error, Page},
    },
    infrastructure::adapters::repositories::dynamodb_pagination::{Item, collect_page},
};

pub struct DynamodbExerciseHistoryRepository {
    client: Client,
    table_name: String,
}

impl DynamodbExerciseHistoryRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

fn history_key(user_id: &str, id: &str) -> Item {
    HashMap::from([
        (
            "user_id".to_string(),
            AttributeValue::S(user_id.to_string()),
        ),
        ("sk".to_string(), AttributeValue::S(id.to_string())),
    ])
}

fn sk_cursor(item: &Item) -> Option<String> {
    item.get("sk").and_then(|sk| sk.as_s().ok()).cloned()
}

#[async_trait]
impl ExerciseHistoryRepository for DynamodbExerciseHistoryRepository {
    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<ExerciseHistory>, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(history_key(user_id, id)))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get exercise history"))?;

        output
            .item
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get exercise history"))
            })
            .transpose()
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        exercise_id: Option<&str>,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<ExerciseHistory>, Error> {
        let start_key = cursor.map(|id| history_key(user_id, id));

        let page = collect_page(limit, start_key, sk_cursor, |start_key| {
            let mut request = self
                .client
                .query()
                .table_name(&self.table_name)
                .key_condition_expression("user_id = :user_id")
                .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
                .scan_index_forward(false)
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            if let Some(exercise_id) = exercise_id {
                request = request
                    .filter_expression("exercise_id = :exercise_id")
                    .expression_attribute_values(
                        ":exercise_id",
                        AttributeValue::S(exercise_id.to_string()),
                    );
            }

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get exercise history"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get exercise history"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
    }

    async fn save(&self, history: &ExerciseHistory) -> Result<(), Error> {
        let mut item: Item = serde_dynamo::to_item(history)
            .map_err(|_| Error::internal_server("Could not save exercise history"))?;

        item.insert("sk".to_string(), AttributeValue::S(history.id.clone()));

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save exercise history"))?;

        Ok(())
    }

    async fn delete(&self, user_id: &str, id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .set_key(Some(history_key(user_id, id)))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete exercise history"))?;

        Ok(())
    }
}
