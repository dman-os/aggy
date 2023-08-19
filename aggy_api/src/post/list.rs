use crate::interlude::*;

use crate::utils::*;

use super::Post;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub enum PostSortingField {
    CreatedAt,
    UpdatedAt,
}

impl SortingField for PostSortingField {
    #[inline]
    fn sql_field_name(&self) -> String {
        match self {
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ListPosts;

common::list_request!(PostSortingField);

#[derive(Debug, thiserror::Error, serde::Serialize, utoipa::ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    // #[error("access denied")]
    // AccessDenied,
    #[error("invalid input: {issues:?}")]
    InvalidInput {
        #[from]
        issues: ValidationErrors,
    },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

common::list_response!(Post);

fn validate_request(
    request: Request,
) -> Result<(String, PostSortingField, SortingOrder, Option<String>), validator::ValidationErrors> {
    validator::Validate::validate(&request)?;

    if request.after_cursor.is_some() || request.before_cursor.is_some() {
        let (is_after, cursor) = request
            .after_cursor
            .map(|cursor| (true, cursor))
            .or_else(|| Some((false, request.before_cursor.unwrap())))
            .unwrap();
        let invalid_cursor_err = |msg: &'static str| {
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                if is_after {
                    "afterCursor"
                } else {
                    "beforeCursor"
                },
                validator::ValidationError {
                    code: "invalid_cursor".into(),
                    message: Some(msg.into()),
                    params: [(std::borrow::Cow::from("value"), serde_json::json!(cursor))]
                        .into_iter()
                        .collect(),
                },
            );
            issues
        };
        let cursor: Cursor<serde_json::Value, PostSortingField> = cursor
            .parse()
            .map_err(|_| invalid_cursor_err("unable to decode cursor"))?;
        // let op = match (cursor.order, is_after) {
        //     (SortingOrder::Ascending, true) | (SortingOrder::Descending, false) => ">",
        //     (SortingOrder::Ascending, false) | (SortingOrder::Descending, true) => "<",
        // };
        let op = if is_after { ">" } else { "<" };
        // FIXME: sql injection, consider HMACing cursors
        let clause = match cursor.field {
            PostSortingField::CreatedAt | PostSortingField::UpdatedAt => {
                let arr = cursor
                    .value
                    .as_array()
                    .ok_or_else(|| invalid_cursor_err("nonsensical cursor"))?;
                let value = arr[0]
                    .as_i64()
                    .ok_or_else(|| invalid_cursor_err("nonsensical cursor"))?;
                let id = arr[1]
                    .as_str()
                    .ok_or_else(|| invalid_cursor_err("nonsensical cursor"))?;
                let column = cursor.field.sql_field_name();
                format!(
                    r#"WHERE 
                        {column} {op} (TO_TIMESTAMP({value}) AT TIME ZONE 'UTC')
                        AND 
                        id {} $guessme${id}$guessme$::UUID
                        "#,
                    if is_after { "<" } else { ">" }
                )
            }
        };
        Ok((clause, cursor.field, cursor.order, cursor.filter))
    } else {
        Ok((
            "".into(),
            request.sorting_field.unwrap_or(PostSortingField::CreatedAt),
            request.sorting_order.unwrap_or(SortingOrder::Descending),
            request.filter,
        ))
    }
}

#[async_trait::async_trait]
impl crate::Endpoint for ListPosts {
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
        let crate::Db::Pg { db_pool } = &cx.db /* else {
            return Err(Error::Internal{message: "this endpoint is not implemented for this db".to_string()});
        } */;
        let limit = request.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let (cursor_clause, sorting_field, sorting_order, filter) =
            validate_request(request).map_err(ValidationErrors::from)?;

        let (sorting_field_str, sorting_order_str) =
            (sorting_field.sql_field_name(), sorting_order.sql_key_word());
        let result = sqlx::query(
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
    FROM (
        posts.posts as p
            LEFT JOIN
        auth.users as u
            ON (p.author_id = u.id)
    ) 
    WHERE cast($1 as text) IS NULL OR (
        u.username ILIKE $1
    )
    ORDER BY {sorting_field_str}, id {sorting_order_str}
    NULLS LAST
) as f
{cursor_clause}
-- fetch one more to check if we have more data 
-- (counts are expensive or something)
LIMIT $2 + 1 
        "#
            )
            .as_str(),
        )
        .bind(filter.as_ref())
        .bind(limit as i64)
        .fetch_all(db_pool)
        .await;
        match result {
            Err(sqlx::Error::RowNotFound) => Ok(Response {
                cursor: None,
                items: vec![],
            }),
            Err(err) => Err(common::internal_err!("db err: {err}")),
            Ok(rows) => {
                use sqlx::FromRow;
                let more_rows_pending = rows.len() == limit + 1;
                // map rows to structs
                let items = rows
                    .iter()
                    .take(limit as _)
                    .map(Post::from_row)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|err| Error::Internal {
                        message: format!("row mapping err: {err}"),
                    })?;
                // construct cursor if necessary
                let cursor = if more_rows_pending {
                    Some(
                        Cursor {
                            value: {
                                let last = items.last().unwrap();
                                match sorting_field {
                                    PostSortingField::CreatedAt => {
                                        serde_json::json!([
                                            last.created_at.unix_timestamp(),
                                            last.id
                                        ])
                                    }
                                    PostSortingField::UpdatedAt => {
                                        serde_json::json!([
                                            last.updated_at.unix_timestamp(),
                                            last.id
                                        ])
                                    }
                                }
                            },
                            field: sorting_field,
                            order: sorting_order,
                            filter,
                        }
                        .to_encoded_str(),
                    )
                } else {
                    None
                };
                Ok(Response { cursor, items })
            }
        }
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            InvalidInput { .. } => Self::BAD_REQUEST,
            // AccessDenied => Self::UNAUTHORIZED,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for ListPosts {
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/posts";

    type SharedCx = SharedContext;
    type HttpRequest = (Query<Request>, DiscardBody);

    fn request((Query(request), _): Self::HttpRequest) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            // auth_token: Some(token),
            ..request
        })
    }

    fn response(resp: Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for ListPosts {
    const TAG: &'static crate::Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [Response {
            cursor: None,
            items: vec![
                Post {
                    id: default(),
                    created_at: OffsetDateTime::now_utc(),
                    updated_at: OffsetDateTime::now_utc(),
                    epigram_id: "f26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61"
                        .into(),
                    title: "Earth 2 reported to begin operations next circumsolar year".into(),
                    url: Some("ipns://𝕏.com/stella_oort/48723494675897134423".into()),
                    body: Some("Please sign in to see this xeet.".into()),
                    author_username: "tazental".into(),
                    author_pic_url: None,
                    author_pub_key:
                        "f196b70071ff6d9c6480677814ac78d2d1478a05a46c60d1dcd7afd21befb0b89".into(),
                    epigram: None,
                },
                Post {
                    id: Default::default(),
                    created_at: time::OffsetDateTime::now_utc(),
                    updated_at: time::OffsetDateTime::now_utc(),
                    epigram_id: "f26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61"
                        .into(),
                    title: "Earth 2 reported to begin operations next circumsolar year".into(),
                    url: Some("ipns://𝕏.com/stella_oort/48723494675897134423".into()),
                    body: Some("Please sign in to see this xeet.".into()),
                    author_username: "tazental".into(),
                    author_pic_url: None,
                    author_pub_key:
                        "f196b70071ff6d9c6480677814ac78d2d1478a05a46c60d1dcd7afd21befb0b89".into(),
                    epigram: None,
                },
            ],
        }]
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Error>> {
        vec![
            // ("Access denied", Error::AccessDenied),
            (
                "Invalid input",
                Error::InvalidInput {
                    issues: {
                        let mut issues = validator::ValidationErrors::new();
                        issues.add(
                            "limit",
                            validator::ValidationError {
                                code: std::borrow::Cow::from("range"),
                                message: None,
                                params: [(std::borrow::Cow::from("value"), serde_json::json!(0))]
                                    .into_iter()
                                    .collect(),
                            },
                        );
                        issues.into()
                    },
                },
            ),
            (
                "Internal server error",
                Error::Internal {
                    message: "internal server error".to_string(),
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    // TODO: rigourous test suite, this module's not the most type safe

    use crate::interlude::*;

    use crate::post::list::*;
    use crate::user::testing::*;

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "limit": 25,
            "filter": USER_01_USERNAME,
            "sortingField": "updatedAt",
            "sortingOrder": "descending"
        })
    }

    common::table_tests! {
        list_posts_validate,
        (request, err_field),
        {
            match crate::post::list::validate_request(request) {
                Ok(_) => {
                    if let Some(err_field) = err_field {
                        panic!("validation succeeded, was expecting err on field: {err_field}");
                    }
                }
                Err(err) => {
                    let err_field = err_field.expect("unexpected validation failure");
                    if !err.field_errors().contains_key(&err_field) {
                        panic!("validation didn't fail on expected field: {err_field}, {err:?}");
                    }
                }
            }
        }
    }

    list_posts_validate! {
        rejects_too_large_limits: (
            Request {
                limit: Some(99999),
                ..fixture_request()
            },
            Some("limit"),
        ),
        rejects_both_cursors_at_once: (
            Request {
                before_cursor: Some("cursorstr".into()),
                after_cursor: Some("cursorstr".into()),
                auth_token: None,
                limit: None,
                filter: None,
                sorting_field: None,
                sorting_order: None,
            },
            Some("__all__"),
        ),
        rejects_cursors_with_filter: (
            Request {
                after_cursor: Some("cursorstr".into()),
                ..fixture_request()
            },
            Some("__all__"),
        ),
    }

    macro_rules! list_posts_integ {
        ($(
            $name:ident: {
                uri: $uri:expr,
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
                            router: crate::post::router(),
                            cx_fn: crate::utils::testing::cx_fn,
                            $(check_json: $check_json,)?
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    list_posts_integ! {
        works: {
            uri: format!("/posts?limit=2"),
            status: StatusCode::OK,
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn(test_cx);
                    let resp_body_json = response_json.unwrap();
                    assert_eq!(resp_body_json["items"].as_array().unwrap().len(), 2);
                    assert!(resp_body_json["cursor"].as_str().is_some());
                    let app = crate::post::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(
                                    format!(
                                        "/posts?afterCursor={}",
                                        resp_body_json["cursor"].as_str().unwrap()
                                    )
                                )
                                .header(http::header::CONTENT_TYPE, "application/json")
                                .body(default())
                                .unwrap_or_log(),
                        )
                        .await
                        .unwrap_or_log();
                    let (head, body) = resp.into_parts();
                    let body = hyper::body::to_bytes(body).await.unwrap_or_log();
                    let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_log();
                    assert_eq!(head.status, StatusCode::OK, "{head:?} {body:?}");
                    assert_eq!(
                        body["items"].as_array().unwrap().len(),
                        3,
                        "{head:#?}\n{body:#?}\n{resp_body_json:#?}"
                    );
                    assert!(
                        body["cursor"].is_null(),
                        "{head:#?}\n{body:#?}"
                    );
                })
            },
        },
    }
}
