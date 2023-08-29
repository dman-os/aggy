use crate::interlude::*;

use super::Gram;

use axum::extract::Query;
use sqlx::FromRow;

#[derive(Clone, Copy, Debug)]
pub struct GetGram;

#[derive(Debug)]
pub struct Request {
    // pub auth_token: BearerToken,
    pub id: String,
    pub include_replies: bool,
}

pub type Response = Ref<Gram>;

#[derive(Debug, thiserror::Error, Serialize, ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    #[error("gram not found at id: {id:?}")]
    NotFound { id: String },
    // #[error("{self:?}")]
    // AccessDenied,
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl Endpoint for GetGram {
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
        let id_byte =
            common::utils::decode_hex_multibase(&request.id).map_err(|_| Error::NotFound {
                id: request.id.clone(),
            })?;

        let out = match &cx.db {
            crate::Db::Pg { db_pool } => {
                if request.include_replies {
                    let rows = sqlx::query(
                        r#"
WITH RECURSIVE recurs AS (
    SELECT *
    FROM grams.grams
    WHERE id = $1
        UNION
    SELECT g.*
    FROM 
        grams.grams g
            INNER JOIN
        recurs
            ON g.parent_id = recurs.id
) SELECT 
    util.multibase_encode_hex(recurs.id) as "id"
    ,created_at
    ,content
    ,coty
    ,util.multibase_encode_hex(parent_id) as "parent_id"
    ,util.multibase_encode_hex(sig) as "sig"
    ,util.multibase_encode_hex(author_pubkey) as "author_pubkey"
    ,author_alias
    ,NULL as "reply_count"
FROM recurs 
                "#,
                    )
                    .bind(&id_byte)
                    .fetch_all(db_pool)
                    .await
                    .map_err(|err| match err {
                        sqlx::Error::RowNotFound => Error::NotFound {
                            id: request.id.clone(),
                        },
                        _ => common::internal_err!("db error: {err}"),
                    })?;
                    if rows.is_empty() {
                        return Err(Error::NotFound {
                            id: request.id.clone(),
                        });
                    }

                    type OptVec = Vec<Option<Gram>>;
                    type FilialMap = std::collections::HashMap<String, Vec<usize>>;

                    let mut arr = vec![];
                    let mut filial_map: FilialMap = default();
                    let mut root_idx = None;

                    for row in rows {
                        let item = Gram::from_row(&row)
                            .map_err(|err| common::internal_err!("row mapping error: {err}"))?;
                        if let Some(parent_id) = item.parent_id.as_ref() {
                            if let Some(replies) = filial_map.get_mut(parent_id) {
                                replies.push(arr.len());
                            } else {
                                filial_map.insert(parent_id.clone(), vec![arr.len()]);
                            }
                        }
                        if item.id == request.id {
                            root_idx = Some(arr.len());
                        }
                        arr.push(Some(item))
                    }
                    fn collect_replies(
                        root: &mut Gram,
                        arr: &mut OptVec,
                        filial_map: &mut FilialMap,
                    ) -> Result<(), Error> {
                        let Some(immediate_replys) = filial_map.remove(&root.id[..]) else {
                            return Ok(())
                        };
                        let mut replies = Vec::with_capacity(immediate_replys.len());
                        for idx in immediate_replys {
                            let mut reply = arr[idx].take().expect_or_log("item at index was None");
                            collect_replies(&mut reply, arr, filial_map)?;
                            replies.push(reply);
                        }
                        root.replies = Some(replies);
                        Ok(())
                    }
                    let root_idx =
                        root_idx.expect_or_log("requested gram not present in result set");
                    let mut root = arr[root_idx].take().expect_or_log("item at index was None");
                    collect_replies(&mut root, &mut arr, &mut filial_map)?;
                    debug_assert!(filial_map.is_empty(), "{filial_map:#?}");
                    Gram {
                        reply_count: Some((arr.len() - 1) as i64),
                        ..root
                    }
                } else {
                    let row = sqlx::query!(
                        r#"
SELECT 
    util.multibase_encode_hex(id) as "id!"
    ,created_at
    ,content
    ,coty
    ,util.multibase_encode_hex(parent_id) as "parent_id?"
    ,util.multibase_encode_hex(sig) as "sig!"
    ,util.multibase_encode_hex(author_pubkey) as "author_pubkey!"
    ,author_alias as "author_alias?"
    ,(
        WITH RECURSIVE recurs AS (
            SELECT id
            FROM grams.grams
            WHERE id = $1
                UNION
            SELECT g.id
            FROM 
                grams.grams g
                    INNER JOIN
                recurs
                    ON g.parent_id = recurs.id
        )
        SELECT COUNT(1) FROM recurs
    ) as "reply_count"
FROM grams.grams 
WHERE id = $1
                "#,
                        &id_byte
                    )
                    .fetch_one(db_pool)
                    .await
                    .map_err(|err| match err {
                        sqlx::Error::RowNotFound => Error::NotFound { id: request.id },
                        err => panic!("db error: {err}"),
                    })?;
                    Gram {
                        id: row.id,
                        created_at: row.created_at,
                        content: row.content,
                        coty: row.coty,
                        parent_id: row.parent_id,
                        author_pubkey: row.author_pubkey,
                        author_alias: row.author_alias,
                        sig: row.sig,
                        reply_count: Some(row.reply_count.unwrap().saturating_sub(1)),
                        replies: default(),
                    }
                }
            }
        };
        Ok(out.into())
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            NotFound { .. } => Self::NOT_FOUND,
            // AccessDenied => Self::UNAUTHORIZED,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Deserialize, utoipa::IntoParams)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(default)]
    include_replies: bool,
}

impl HttpEndpoint for GetGram {
    type SharedCx = SharedContext;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/grams/:id";

    type HttpRequest = (Query<QueryParams>, Path<String>, DiscardBody);

    fn request(
        (Query(params), Path(id), _): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            /*auth_token, */ id,
            include_replies: params.include_replies,
            // include_replies: true,
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for GetGram {
    const TAG: &'static Tag = &crate::gram::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [super::testing::GRAM_01.clone()]
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Error>> {
        use Error::*;
        vec![
            // ("Access Denied", AccessDenied),
            (
                "Not Found",
                NotFound {
                    id: "asldkfjaslkdfja".into(),
                },
            ),
            (
                "Internal server error",
                Internal {
                    message: "internal server error".into(),
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    use crate::gram::testing::*;

    macro_rules! get_gram_integ {
        ($(
            $name:ident: {
                uri: $uri:expr,
                // auth_token: $auth_token:expr,
                status: $status:expr,
                $(check_json: $check_json:expr,)?
                $(extra_assertions: $extra_fn:expr,)?
            },
        )*) => {
            mod integ {
                use super::*;
                common::integration_table_tests! {
                    $(
                        $name: {
                            uri: $uri,
                            method: "GET",
                            status: $status,
                            router: crate::gram::router(),
                            cx_fn: crate::utils::testing::cx_fn,
                            $(check_json: $check_json,)?
                            // auth_token: $auth_token,
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    get_gram_integ! {
        works_includes_replies: {
            uri: format!("/grams/{GRAM_01_ID}?includeReplies=true"),
            // auth_token: SERVICE.into(),
            status: StatusCode::OK,
            check_json: serde_json::json!(*GRAM_01).remove_keys_from_obj(&["createdAt", "replyCount"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let resp_body_json = response_json.unwrap();
                    assert!(resp_body_json["replies"].is_array());
                    assert_eq!(resp_body_json["replyCount"].as_i64(), Some(6));
                })
            },
        },
        works_excludes_replies: {
            uri: format!("/grams/{GRAM_01_ID}"),
            // auth_token: SERVICE.into(),
            status: StatusCode::OK,
            check_json: serde_json::json!(*GRAM_01).remove_keys_from_obj(&["createdAt", "replyCount"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let resp_body_json = response_json.unwrap();
                    assert!(resp_body_json["replies"].is_null());
                    assert_eq!(resp_body_json["replyCount"].as_i64(), Some(6));
                })
            },
        },
        fails_if_not_found: {
            uri: format!("/grams/{}", Uuid::new_v4()),
            status: StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound",
            }),
        },
        fails_if_not_found_2: {
            uri: format!("/grams/{}?includeReplies=true", Uuid::new_v4()),
            status: StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound",
            }),
        },
    }
}
