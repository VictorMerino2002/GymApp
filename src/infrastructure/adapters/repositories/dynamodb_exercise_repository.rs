use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::ExerciseRepository,
    domain::{
        entities::Exercise,
        value_objects::{Error, ExerciseCategory, Muscle, Page},
    },
    infrastructure::adapters::repositories::{
        dynamodb_batch::batch_get_by_ids,
        dynamodb_pagination::{collect_page, id_cursor, id_key, muscle_value},
    },
};

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
        let items = batch_get_by_ids(
            &self.client,
            &self.table_name,
            ids,
            "Could not get exercises",
        )
        .await?;

        serde_dynamo::from_items(items)
            .map_err(|_| Error::internal_server("Could not get exercises"))
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
        muscle: Option<Muscle>,
        category: Option<ExerciseCategory>,
    ) -> Result<Page<Exercise>, Error> {
        let muscle = muscle.map(muscle_value).transpose()?;
        let category_value = category
            .map(serde_dynamo::to_attribute_value::<_, AttributeValue>)
            .transpose()
            .map_err(|_| Error::internal_server("Could not build category filter"))?;

        let mut conditions = Vec::new();
        if muscle.is_some() {
            conditions.push("contains(#muscles, :muscle)");
        }
        match category {
            Some(ExerciseCategory::Strength) => {
                conditions.push("(#category = :category OR attribute_not_exists(#category))")
            }
            Some(_) => conditions.push("#category = :category"),
            None => {}
        }
        let filter_expression = conditions.join(" AND ");

        let page = collect_page(limit, cursor.map(id_key), id_cursor, |start_key| {
            let mut request = self
                .client
                .scan()
                .table_name(&self.table_name)
                .limit(i32::from(limit))
                .set_exclusive_start_key(start_key);

            if !filter_expression.is_empty() {
                request = request.filter_expression(&filter_expression);
            }

            if let Some(muscle) = &muscle {
                request = request
                    .expression_attribute_names("#muscles", "muscles")
                    .expression_attribute_values(":muscle", muscle.clone());
            }

            if let Some(category) = &category_value {
                request = request
                    .expression_attribute_names("#category", "category")
                    .expression_attribute_values(":category", category.clone());
            }

            async move {
                let output = request
                    .send()
                    .await
                    .map_err(|_| Error::internal_server("Could not get exercises"))?;
                Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
            }
        })
        .await?;

        let items = serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get exercises"))?;

        Ok(Page {
            items,
            next_cursor: page.next_cursor,
        })
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
