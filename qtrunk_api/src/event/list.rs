use crate::interlude::*;

use super::{Event, Filter};

#[derive(Clone, Copy, Debug)]
pub struct ListEvents;

pub type Request = Vec<Filter>;

pub type Response = Vec<Event>;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("invalid: {issues}")]
    InvalidInput {
        #[from]
        issues: ValidationErrors,
    },
    #[error("error: internal server error: {message}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl crate::Endpoint for ListEvents {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    #[tracing::instrument(skip(cx))]
    async fn handle(
        &self,
        cx: &Self::Cx,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        // let limit = request.limit.unwrap_or(100);
        let items = if request.is_empty() || (request.len() == 1 && request[0].is_empty()) {
            match &cx.db {
                crate::Db::Pg { db_pool } => {
                    let rows = sqlx::query(
                        r#"
SELECT
    encode(id, 'hex') as "id"
    ,encode(pubkey, 'hex') as "pubkey"
    ,created_at
    ,kind
    ,tags
    ,content
    ,encode(sig, 'hex') as "sig"
FROM events
        "#,
                    )
                    .fetch_all(db_pool)
                    .await
                    .unwrap_or_log();

                    rows.iter()
                        .map(Event::from_row)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap_or_log()
                }
            }
        } else {
            todo!()
        };
        Ok(items)
        /* let result = sqlx::query(
                    format!(
                        r#"
        SELECT *
        FROM (
            SELECT
                p.created_at as "created_at"
                ,p.updated_at as "updated_at"
                ,p.id as "id"
                ,p.title as "title"
                ,p.url as "url"
                ,p.body as "body"
                ,util.multibase_encode_hex(p.epigram_id) as "epigram_id"
                ,util.multibase_encode_hex(u.pub_key) as "author_pub_key"
                ,u.username::TEXT as "author_username"
                ,u.pic_url as "author_pic_url"
            FROM events
            WHERE cast($1 as text) IS NULL OR (
                u.username ILIKE $1
            )
            ORDER BY {sorting_field_str}, id {sorting_order_str}
            NULLS LAST
        ) as f
        {cursor_clause}
        -- fetch one more to check if we have more data
        -- (counts are expensive or something)
        LIMIT $2
                "#
                    )
                    .as_str(),
                )
                .bind(filter.as_ref())
                .bind(limit as i64)
                .fetch_all(db_pool)
                .await; */
        /* match result {
            Err(sqlx::Error::RowNotFound) => Ok(vec![]),
            Err(err) => Err(common::internal_err!("db err: {err}")),
            Ok(rows) => {
                use sqlx::FromRow;
                let items = rows
                    .iter()
                    .take(limit as _)
                    .map(Event::from_row)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|err| common::internal_err!("row mapping err: {err}"))?;
                Ok(items)
            }
        } */
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    use super::Request;
    use crate::event::testing::*;

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fixture_request_json() -> serde_json::Value {
        json!([])
    }

    #[tokio::test]
    async fn works() -> eyre::Result<()> {
        common::utils::testing::setup_tracing_once();
        let (testing, cx) = crate::utils::testing::cx_fn(common::function_full!()).await;
        {
            let filter = fixture_request_json();
            let filter = serde_json::from_value(filter).unwrap();
            let ok = crate::event::list::ListEvents.handle(&cx, filter).await?;
            assert_eq!(ok.len(), 5, "{ok:?}");
        }
        testing.close().await;
        Ok(())
    }

    common::table_tests! {
        integ tokio,
        (request_json, expected_json),
        {
            let (testing, cx) = crate::utils::testing::cx_fn(common::function_full!()).await;
            {
                let filter = serde_json::from_value(request_json).unwrap();
                let ok = match crate::event::list::ListEvents.handle(&cx, filter).await{
                    Ok(value) => json!(value),
                    Err(value) => json!(value),
                };
                tracing::info!(?ok);
                check_json(
                    ("expected", &expected_json),
                    ("response", &ok),
                );
            }
            testing.close().await;
        },
        multi_thread: true,
    }

    /* integ! {
        supports_id_filter: (
            json!({
                "ids": [EVENT_03_ID]
            }),
            json!(*EVENT_03_ID)
        ),
    } */
}
