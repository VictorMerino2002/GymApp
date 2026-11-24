use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use aws_sdk_dynamodb::{
    Client,
    types::{AttributeValue, DeleteRequest, KeysAndAttributes, WriteRequest},
};

use crate::{
    domain::value_objects::Error,
    infrastructure::adapters::repositories::dynamodb_pagination::{Item, id_key},
};

const BATCH_GET_MAX_KEYS: usize = 100;
const BATCH_WRITE_MAX_ITEMS: usize = 25;
const BATCH_MAX_ATTEMPTS: u32 = 5;

pub async fn batch_get_by_ids(
    client: &Client,
    table_name: &str,
    ids: &[String],
    error_message: &str,
) -> Result<Vec<Item>, Error> {
    let unique_ids: Vec<&String> = ids.iter().collect::<BTreeSet<_>>().into_iter().collect();
    let mut items = Vec::with_capacity(unique_ids.len());

    for chunk in unique_ids.chunks(BATCH_GET_MAX_KEYS) {
        let keys_and_attributes = KeysAndAttributes::builder()
            .set_keys(Some(chunk.iter().map(|id| id_key(id)).collect()))
            .build()
            .map_err(|_| Error::internal_server(error_message))?;

        let mut request_items = HashMap::from([(table_name.to_string(), keys_and_attributes)]);
        let mut attempt = 0;

        loop {
            let output = client
                .batch_get_item()
                .set_request_items(Some(request_items))
                .send()
                .await
                .map_err(|_| Error::internal_server(error_message))?;

            items.extend(
                output
                    .responses
                    .and_then(|mut responses| responses.remove(table_name))
                    .unwrap_or_default(),
            );

            match output.unprocessed_keys.filter(|keys| !keys.is_empty()) {
                None => break,
                Some(unprocessed) => {
                    attempt += 1;
                    if attempt >= BATCH_MAX_ATTEMPTS {
                        return Err(Error::internal_server(error_message));
                    }
                    tokio::time::sleep(Duration::from_millis(50 * 2u64.pow(attempt))).await;
                    request_items = unprocessed;
                }
            }
        }
    }

    Ok(items)
}

pub async fn batch_delete_keys(
    client: &Client,
    table_name: &str,
    keys: Vec<Item>,
    error_message: &str,
) -> Result<(), Error> {
    for chunk in keys.chunks(BATCH_WRITE_MAX_ITEMS) {
        let requests = chunk
            .iter()
            .map(|key| {
                let delete = DeleteRequest::builder()
                    .set_key(Some(key.clone()))
                    .build()
                    .map_err(|_| Error::internal_server(error_message))?;
                Ok(WriteRequest::builder().delete_request(delete).build())
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let mut request_items = HashMap::from([(table_name.to_string(), requests)]);
        let mut attempt = 0;

        loop {
            let output = client
                .batch_write_item()
                .set_request_items(Some(request_items))
                .send()
                .await
                .map_err(|_| Error::internal_server(error_message))?;

            match output.unprocessed_items.filter(|items| !items.is_empty()) {
                None => break,
                Some(unprocessed) => {
                    attempt += 1;
                    if attempt >= BATCH_MAX_ATTEMPTS {
                        return Err(Error::internal_server(error_message));
                    }
                    tokio::time::sleep(Duration::from_millis(50 * 2u64.pow(attempt))).await;
                    request_items = unprocessed;
                }
            }
        }
    }

    Ok(())
}

pub async fn delete_partition(
    client: &Client,
    table_name: &str,
    index_name: Option<&str>,
    partition_attribute: &str,
    partition_value: &str,
    key_attributes: [&str; 2],
    error_message: &str,
) -> Result<(), Error> {
    let keys = client
        .query()
        .table_name(table_name)
        .set_index_name(index_name.map(str::to_string))
        .key_condition_expression("#pk = :pk")
        .projection_expression("#k0, #k1")
        .expression_attribute_names("#pk", partition_attribute)
        .expression_attribute_names("#k0", key_attributes[0])
        .expression_attribute_names("#k1", key_attributes[1])
        .expression_attribute_values(":pk", AttributeValue::S(partition_value.to_string()))
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await
        .map_err(|_| Error::internal_server(error_message))?;

    batch_delete_keys(client, table_name, keys, error_message).await
}
