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

struct Args {
    ctr: usize,
    inner: sqlx::postgres::PgArguments,
}
impl Args {
    fn add<'q, T>(&mut self, val: T) -> String
    where
        T: sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + 'static + Send + Sync,
    {
        use sqlx::Arguments;
        self.inner.add(val);
        self.ctr += 1;
        format!("${ctr}", ctr = self.ctr)
    }
}

fn pg_query(request: &Request) -> Result<(String, sqlx::postgres::PgArguments), Error> {
    let mut args = Args {
        ctr: default(),
        inner: default(),
    };
    /* let mut rng = rand::thread_rng();
    let gen_border = || {
        use rand::*;
        const ABET: &[u8] = b"ABCDEFGHIJKLMNOPabcdefghijklmnop";
        let idx: [usize; 10] = rng.gen();
        let mut out = [0u8; 10];
        for ii in idx {
            out[ii] = ABET[ii % ABET.len()];
        }
        out
    }; */

    let mut limit = None;
    let where_clause = request
        .iter()
        // look for and store the max limit among the filters
        .inspect(|filter| match (limit, filter.limit) {
            (None, Some(_)) => {
                limit = filter.limit;
            }
            (Some(cur), Some(fl)) => limit = Some(cur.max(fl)),
            _ => {}
        })
        // we want the index for error reporting purposes
        .enumerate()
        .map(|(idx, filter)| {
            // map each filter option to a clause
            [
                filter.ids.as_deref().map(|items| {
                    let items = items
                        .iter()
                        .map(|hex| {
                            data_encoding::HEXLOWER_PERMISSIVE
                                .decode(hex.as_bytes())
                                .map_err(|_| hex)
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|hex| {
                            let mut issues = validator::ValidationErrors::new();
                            issues.add(
                                "ids",
                                validator::ValidationError {
                                    code: "invalid_hex".into(),
                                    message: Some("error decoding hex values in `ids`".into()),
                                    params: [
                                        (std::borrow::Cow::from("value"), serde_json::json!(hex)),
                                        (
                                            std::borrow::Cow::from("filter_idx"),
                                            serde_json::json!(idx),
                                        ),
                                    ]
                                    .into_iter()
                                    .collect(),
                                },
                            );
                            issues
                        })
                        .map_err(ValidationErrors::from)?;
                    let ph = args.add(items);
                    Ok::<_, Error>(format!("id = ANY ({ph})"))
                }),
                filter.authors.as_deref().map(|items| {
                    let items = items
                        .iter()
                        .map(|hex| {
                            data_encoding::HEXLOWER_PERMISSIVE
                                .decode(hex.as_bytes())
                                .map_err(|_| hex)
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|hex| {
                            let mut issues = validator::ValidationErrors::new();
                            issues.add(
                                "authors",
                                validator::ValidationError {
                                    code: "invalid_hex".into(),
                                    message: Some("error decoding hex values in `authors`".into()),
                                    params: [
                                        (std::borrow::Cow::from("value"), serde_json::json!(hex)),
                                        (
                                            std::borrow::Cow::from("filter_idx"),
                                            serde_json::json!(idx),
                                        ),
                                    ]
                                    .into_iter()
                                    .collect(),
                                },
                            );
                            issues
                        })
                        .map_err(ValidationErrors::from)?;
                    let ph = args.add(items);
                    Ok::<_, Error>(format!("pubkey = ANY ({ph})"))
                }),
                filter.kinds.as_deref().map(|items| {
                    let items = items.iter().map(|kind| *kind as i64).collect::<Vec<_>>();
                    let ph = args.add(items);
                    Ok::<_, Error>(format!("kind = ANY ({ph})"))
                }),
                filter.since.map(|ts| {
                    let ph = args.add(ts);
                    Ok::<_, Error>(format!("created_at > ({ph})"))
                }),
                filter.until.map(|ts| {
                    let ph = args.add(ts);
                    Ok::<_, Error>(format!("created_at < ({ph})"))
                }),
                filter.tags.as_ref().map(|tags| {
                    Ok::<_, Error>(
                        tags.iter()
                            .map(|(char, needles)| {
                                let jsonpaths = needles
                                    .iter()
                                    .map(|val| {
                                        format!(r#"$ ? (@[0] == "{char}" && @[1] == "{val}")"#)
                                    })
                                    .collect::<Vec<_>>();
                                let ph = args.add(dbg!(jsonpaths));
                                format!("tags @? ANY({ph}::jsonpath[])")
                            })
                            .fold("true".to_string(), |mut acc, val| {
                                acc.push_str(" AND (");
                                acc.push_str(&val[..]);
                                acc.push_str(")");
                                acc
                            }),
                    )
                }), // TODO: tags
            ]
            .into_iter()
            // remove all the None values
            .flatten()
            // TODO: is it possible to avoid allocation to a Vec here?
            .collect::<Result<Vec<_>, _>>()
            .map(|clauses| {
                clauses
                    .into_iter()
                    // combine the clauses using AND
                    .fold("true".to_string(), |mut acc, val| {
                        acc.push_str(" AND (");
                        acc.push_str(&val[..]);
                        acc.push_str(")");
                        acc
                    })
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|cl| !cl.is_empty())
        .fold("WHERE false".to_string(), |mut acc, val| {
            acc.push_str(" OR (");
            acc.push_str(&val[..]);
            acc.push_str(")");
            acc
        });

    let where_clause = if where_clause == "WHERE false" {
        String::new()
    } else {
        where_clause
    };

    let limit = limit.unwrap_or(100).min(100) as i64;
    let limit = args.add(limit);
    Ok((
        format!(
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
{where_clause}
ORDER BY created_at
LIMIT {limit}
        "#,
        ),
        args.inner,
    ))
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
        let items = match &cx.db {
            crate::Db::Pg { db_pool } => {
                let (query, args) = pg_query(&request)?;
                sqlx::query_as_with(&query[..], args)
                    .fetch_all(db_pool)
                    .await
                    .unwrap_or_log()
            }
        };
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    // use super::Request;
    use crate::event::testing::*;

    /*
    // TODO: serialization and validation tests for Filter
    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }
    */

    fn fixture_request_json() -> serde_json::Value {
        json!([{}])
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

    integ! {
        supports_id_filter: (
            json!([{
                "ids": [EVENT_03.id, "dc24085d327469c5151e966df7c7766b626ed9064de6fa7b9a0b0a66fbde002a"]
            }]),
            json!([*EVENT_03])
        ),
        supports_author_filter: (
            json!([{
                "authors": [EVENT_01.pubkey, "dc24085d327469c5151e966df7c7766b626ed9064de6fa7b9a0b0a66fbde002a"]
            }]),
            json!([*EVENT_01])
        ),
        supports_kind_filter: (
            json!([{
                "kinds": [0, 42069]
            }]),
            json!([*EVENT_02])
        ),
        supports_since_filter: (
            json!([{
                "since":  EVENT_03.created_at.unix_timestamp()
            }]),
            json!([*EVENT_04, *EVENT_05])
        ),
        supports_until_filter: (
            json!([{
                "until":  EVENT_03.created_at.unix_timestamp()
            }]),
            json!([*EVENT_01, *EVENT_02])
        ),
        supports_tag_filter: (
            json!([{
                "#e": [EVENT_03.id]
            }]),
            json!([*EVENT_04, *EVENT_05])
        ),
    }
}
