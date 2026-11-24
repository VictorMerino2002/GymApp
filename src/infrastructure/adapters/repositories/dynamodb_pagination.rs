use std::{collections::HashMap, future::Future};

use aws_sdk_dynamodb::types::AttributeValue;

use crate::domain::value_objects::{Error, Muscle};

pub type Item = HashMap<String, AttributeValue>;

pub struct RawPage {
    pub items: Vec<Item>,
    pub next_cursor: Option<String>,
}

pub fn id_key(id: &str) -> Item {
    HashMap::from([("id".to_string(), AttributeValue::S(id.to_string()))])
}

pub fn id_cursor(item: &Item) -> Option<String> {
    item.get("id").and_then(|id| id.as_s().ok()).cloned()
}

pub fn muscle_value(muscle: Muscle) -> Result<AttributeValue, Error> {
    serde_dynamo::to_attribute_value(muscle)
        .map_err(|_| Error::internal_server("Could not build muscle filter"))
}

pub async fn collect_page<F, Fut, C>(
    limit: u16,
    start_key: Option<Item>,
    cursor_of: C,
    mut fetch: F,
) -> Result<RawPage, Error>
where
    F: FnMut(Option<Item>) -> Fut,
    Fut: Future<Output = Result<(Vec<Item>, Option<Item>), Error>>,
    C: Fn(&Item) -> Option<String>,
{
    let limit = usize::from(limit);
    let mut items = Vec::with_capacity(limit);
    let mut start_key = start_key;

    loop {
        let (page, last_key) = fetch(start_key).await?;
        items.extend(page);

        if items.len() >= limit {
            let truncated = items.len() > limit;
            items.truncate(limit);

            let next_cursor = if truncated || last_key.is_some() {
                items.last().and_then(&cursor_of)
            } else {
                None
            };

            return Ok(RawPage { items, next_cursor });
        }

        match last_key {
            Some(key) => start_key = Some(key),
            None => {
                return Ok(RawPage {
                    items,
                    next_cursor: None,
                });
            }
        }
    }
}
