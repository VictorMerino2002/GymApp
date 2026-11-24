use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use async_trait::async_trait;
use aws_sdk_dynamodb::{
    Client,
    types::{AttributeValue, KeysAndAttributes},
};

use crate::{
    application::ports::repositories::ExerciseRepository,
    domain::{
        entities::Exercise,
        value_objects::{Error, Page},
    },
};

const BATCH_GET_MAX_KEYS: usize = 100;
const BATCH_GET_MAX_ATTEMPTS: u32 = 5;

pub struct DynamodbExerciseRepository {
    client: Client,
    table_name: String,
}

impl DynamodbExerciseRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl ExerciseRepository for DynamodbExerciseRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Exercise>, Error> {
        let output = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not get exercise"))?;

        output
            .item
            .map(|item| {
                serde_dynamo::from_item(item)
                    .map_err(|_| Error::internal_server("Could not get exercise"))
            })
            .transpose()
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<Exercise>, Error> {
        let unique_ids: Vec<&String> = ids.iter().collect::<BTreeSet<_>>().into_iter().collect();
        let mut exercises = Vec::with_capacity(unique_ids.len());

        for chunk in unique_ids.chunks(BATCH_GET_MAX_KEYS) {
            let keys = chunk
                .iter()
                .map(|id| HashMap::from([("id".to_string(), AttributeValue::S(id.to_string()))]))
                .collect();

            let keys_and_attributes = KeysAndAttributes::builder()
                .set_keys(Some(keys))
                .build()
                .map_err(|_| Error::internal_server("Could not get exercises"))?;

            let mut request_items = HashMap::from([(self.table_name.clone(), keys_and_attributes)]);
            let mut attempt = 0;

            loop {
                let output = self
                    .client
                    .batch_get_item()
                    .set_request_items(Some(request_items))
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get exercises"))?;

                let found = output
                    .responses
                    .and_then(|mut responses| responses.remove(&self.table_name))
                    .unwrap_or_default();

                let parsed: Vec<Exercise> = serde_dynamo::from_items(found)
                    .map_err(|_| Error::internal_server("Could not get exercises"))?;
                exercises.extend(parsed);

                match output.unprocessed_keys.filter(|keys| !keys.is_empty()) {
                    None => break,
                    Some(unprocessed) => {
                        attempt += 1;
                        if attempt >= BATCH_GET_MAX_ATTEMPTS {
                            return Err(Error::internal_server("Could not get exercises"));
                        }
                        tokio::time::sleep(Duration::from_millis(50 * 2u64.pow(attempt))).await;
                        request_items = unprocessed;
                    }
                }
            }
        }

        Ok(exercises)
    }

    async fn find_all(&self) -> Result<Vec<Exercise>, Error> {
        let items = self
            .client
            .scan()
            .table_name(&self.table_name)
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await
            .map_err(|_| Error::internal_server("Could not get exercises"))?;

        serde_dynamo::from_items(items)
            .map_err(|_| Error::internal_server("Could not get exercises"))
    }

    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Exercise>, Error> {
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
            .map_err(|_| Error::internal_server("Could not get exercises"))?;

        let items = serde_dynamo::from_items(output.items.unwrap_or_default())
            .map_err(|_| Error::internal_server("Could not get exercises"))?;

        let next_cursor = output
            .last_evaluated_key
            .and_then(|key| key.get("id").and_then(|id| id.as_s().ok()).cloned());

        Ok(Page { items, next_cursor })
    }

    async fn save(&self, exercise: &Exercise) -> Result<(), Error> {
        let item = serde_dynamo::to_item(exercise)
            .map_err(|_| Error::internal_server("Could not save exercise"))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save exercise"))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not delete exercise"))?;

        Ok(())
    }
}
