use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use chrono::{Duration, Utc};

use crate::{
    application::ports::repositories::ActivityRepository,
    domain::{entities::Activity, value_objects::Error},
    infrastructure::adapters::repositories::{
        dynamodb_batch::delete_partition,
        dynamodb_pagination::{Item, collect_page},
    },
};

const ACTIVITY_TTL_DAYS: i64 = 90;

pub struct DynamodbActivityRepository {
    client: Client,
    table_name: String,
}

impl DynamodbActivityRepository {
    pub fn new(client: Client, table_name: impl Into<String>) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }
}

#[async_trait]
impl ActivityRepository for DynamodbActivityRepository {
    async fn save(&self, activity: &Activity) -> Result<(), Error> {
        let mut item: Item = serde_dynamo::to_item(activity)
            .map_err(|_| Error::internal_server("Could not save activity"))?;

        let expires_at = activity.created_at + Duration::days(ACTIVITY_TTL_DAYS);
        item.insert(
            "expires_at".to_string(),
            AttributeValue::N(expires_at.timestamp().to_string()),
        );

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|_| Error::internal_server("Could not save activity"))?;

        Ok(())
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        before: Option<&str>,
        limit: u16,
    ) -> Result<Vec<Activity>, Error> {
        let now = AttributeValue::N(Utc::now().timestamp().to_string());
        let key_condition = match before {
            Some(_) => "user_id = :user_id AND id < :before",
            None => "user_id = :user_id",
        };

        let page = collect_page(
            limit,
            None,
            |_| None,
            |start_key| {
                let mut request = self
                    .client
                    .query()
                    .table_name(&self.table_name)
                    .key_condition_expression(key_condition)
                    .filter_expression("attribute_not_exists(#expires_at) OR #expires_at > :now")
                    .expression_attribute_names("#expires_at", "expires_at")
                    .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
                    .expression_attribute_values(":now", now.clone())
                    .scan_index_forward(false)
                    .limit(i32::from(limit))
                    .set_exclusive_start_key(start_key);

                if let Some(before) = before {
                    request = request.expression_attribute_values(
                        ":before",
                        AttributeValue::S(before.to_string()),
                    );
                }

                async move {
                    let output = request
                        .send()
                        .await
                        .map_err(|_| Error::internal_server("Could not get activities"))?;
                    Ok((output.items.unwrap_or_default(), output.last_evaluated_key))
                }
            },
        )
        .await?;

        serde_dynamo::from_items(page.items)
            .map_err(|_| Error::internal_server("Could not get activities"))
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        delete_partition(
            &self.client,
            &self.table_name,
            None,
            "user_id",
            user_id,
            ["user_id", "id"],
            "Could not delete activities",
        )
        .await
    }
}
