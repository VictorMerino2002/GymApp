use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use aws_sdk_dynamodb::{Client, types::KeysAndAttributes};

use crate::{
    domain::value_objects::Error,
    infrastructure::adapters::repositories::dynamodb_pagination::{Item, id_key},
};

const BATCH_GET_MAX_KEYS: usize = 100;
const BATCH_GET_MAX_ATTEMPTS: u32 = 5;

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
                    if attempt >= BATCH_GET_MAX_ATTEMPTS {
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
