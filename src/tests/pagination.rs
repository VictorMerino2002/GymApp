use std::sync::Mutex;

use crate::infrastructure::adapters::repositories::dynamodb_pagination::{
    Item, RawPage, collect_page, id_cursor, id_key,
};

fn items(ids: &[&str]) -> Vec<Item> {
    ids.iter().map(|id| id_key(id)).collect()
}

fn ids(page: &RawPage) -> Vec<String> {
    page.items.iter().filter_map(id_cursor).collect()
}

async fn paginate(
    limit: u16,
    responses: Vec<(Vec<Item>, Option<Item>)>,
) -> (RawPage, Vec<Option<Item>>) {
    let responses = Mutex::new(responses.into_iter());
    let start_keys = Mutex::new(Vec::new());

    let page = collect_page(limit, None, id_cursor, |start_key| {
        start_keys.lock().unwrap().push(start_key);
        let response = responses.lock().unwrap().next().expect("unexpected fetch");
        async move { Ok(response) }
    })
    .await
    .unwrap();

    (page, start_keys.into_inner().unwrap())
}

#[tokio::test]
async fn last_page_has_no_cursor() {
    let (page, _) = paginate(5, vec![(items(&["a", "b"]), None)]).await;
    assert_eq!(ids(&page), ["a", "b"]);
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn keeps_fetching_until_limit_is_filled() {
    let (page, start_keys) = paginate(
        3,
        vec![
            (items(&["a"]), Some(id_key("a"))),
            (vec![], Some(id_key("x"))),
            (items(&["b", "c"]), Some(id_key("c"))),
        ],
    )
    .await;

    assert_eq!(ids(&page), ["a", "b", "c"]);
    assert_eq!(page.next_cursor.as_deref(), Some("c"));
    assert_eq!(start_keys, [None, Some(id_key("a")), Some(id_key("x"))]);
}

#[tokio::test]
async fn truncates_overflow_and_points_cursor_at_last_kept_item() {
    let (page, _) = paginate(2, vec![(items(&["a", "b", "c"]), None)]).await;
    assert_eq!(ids(&page), ["a", "b"]);
    assert_eq!(page.next_cursor.as_deref(), Some("b"));
}

#[tokio::test]
async fn exact_fill_without_more_data_has_no_cursor() {
    let (page, _) = paginate(2, vec![(items(&["a", "b"]), None)]).await;
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn exact_fill_with_more_data_has_cursor() {
    let (page, _) = paginate(2, vec![(items(&["a", "b"]), Some(id_key("b")))]).await;
    assert_eq!(page.next_cursor.as_deref(), Some("b"));
}
