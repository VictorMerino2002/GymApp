use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use crate::{
    application::ports::repositories::UserRepository,
    domain::{entities::User, value_objects::Error},
};

const USERNAME_INDEX: &str = "username-index";

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

#[async_trait]
impl UserRepository for DynamodbUserRepository {
    async fn save(&self, user: &User) -> Result<(), Error> {
        let item = serde_dynamo::to_item(user)
            .map_err(|_| Error::internal_server("Could not save user"))?;

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
}
