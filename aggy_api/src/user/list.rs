use crate::interlude::*;

use crate::utils::*;

use super::User;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub enum UserSortingField {
    Username,
    Email,
    CreatedAt,
    UpdatedAt,
}

impl SortingField for UserSortingField {
    #[inline]
    fn sql_field_name(&self) -> String {
        match self {
            Self::Username => "username",
            Self::Email => "email",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ListUsers;

common::alias_and_ref!(ListRequest<UserSortingField>, ListUsersRequest, Request, de);

#[derive(Debug, thiserror::Error, serde::Serialize, utoipa::ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    #[error("acess denied")]
    AccessDenied,
    #[error("invalid input: {issues:?}")]
    InvalidInput {
        #[from]
        issues: ValidationErrors,
    },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

crate::impl_from_auth_err!(Error);

common::alias_and_ref!(ListResponse<super::User>, ListUsersResponse, Response, ser);

fn validate_request(
    request: ListRequest<UserSortingField>,
) -> Result<(String, UserSortingField, SortingOrder, Option<String>), validator::ValidationErrors> {
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
        let cursor: Cursor<serde_json::Value, UserSortingField> = cursor
            .parse()
            .map_err(|_| invalid_cursor_err("unable to decode cursor"))?;
        // let op = match (cursor.order, is_after) {
        //     (SortingOrder::Ascending, true) | (SortingOrder::Descending, false) => ">",
        //     (SortingOrder::Ascending, false) | (SortingOrder::Descending, true) => "<",
        // };
        let op = if is_after { ">" } else { "<" };
        // FIXME: sql injection, consider HMACing cursors
        let clause = match cursor.field {
            UserSortingField::Username | UserSortingField::Email => {
                let value = cursor
                    .value
                    .as_str()
                    .ok_or_else(|| invalid_cursor_err("nonsensical cursor"))?;
                let column = cursor.field.sql_field_name();
                format!("WHERE {column} {op} $guessme${value}$guessme$")
            }
            UserSortingField::CreatedAt | UserSortingField::UpdatedAt => {
                let value = cursor
                    .value
                    .as_i64()
                    .ok_or_else(|| invalid_cursor_err("nonsensical cursor"))?;
                let column = cursor.field.sql_field_name();
                format!("WHERE {column} {op} (TO_TIMESTAMP({value}) AT TIME ZONE 'UTC')")
            }
        };
        Ok((clause, cursor.field, cursor.order, cursor.filter))
    } else {
        Ok((
            "".into(),
            request.sorting_field.unwrap_or(UserSortingField::CreatedAt),
            request.sorting_order.unwrap_or(SortingOrder::Descending),
            request.filter,
        ))
    }
}

#[async_trait::async_trait]
impl crate::AuthenticatedEndpoint for ListUsers {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    fn authorize_request(&self, request: &Self::Request) -> crate::auth::authorize::Request {
        crate::auth::authorize::Request {
            auth_token: request.auth_token.clone().unwrap(),
            resource: crate::auth::Resource::Users,
            action: crate::auth::Action::Read,
        }
    }

    // #[tracing::instrument(skip(cx))]
    async fn handle(
        &self,
        cx: &Self::Cx,
        _accessing_user: uuid::Uuid,
        Request(request): Self::Request,
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
SELECT 
    id
    ,created_at
    ,updated_at
    ,email::TEXT as "email"
    ,username::TEXT as "username"
    ,'f' || encode(pub_key, 'hex') as "pub_key"
    ,pic_url
FROM (
    SELECT *
    FROM auth.users
    WHERE cast($1 as text) IS NULL OR (
        username ILIKE '%%' || $1 || '%%'
        OR email ILIKE '%%' || $1 || '%%'
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
            Err(sqlx::Error::RowNotFound) => Ok(ListUsersResponse {
                cursor: None,
                items: vec![],
            }
            .into()),
            Err(err) => Err(Error::Internal {
                message: format!("db err: {err}"),
            }),
            Ok(rows) => {
                use sqlx::FromRow;
                let more_rows_pending = rows.len() == limit + 1;
                // map rows to structs
                let items = rows
                    .iter()
                    .take(limit as _)
                    .map(User::from_row)
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
                                    UserSortingField::Username => serde_json::json!(last.username),
                                    UserSortingField::Email => serde_json::json!(last.email),
                                    UserSortingField::CreatedAt => {
                                        serde_json::json!(last.created_at.unix_timestamp())
                                    }
                                    UserSortingField::UpdatedAt => {
                                        serde_json::json!(last.updated_at.unix_timestamp())
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
                Ok(ListUsersResponse { cursor, items }.into())
            }
        }
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            InvalidInput { .. } => Self::BAD_REQUEST,
            AccessDenied => Self::UNAUTHORIZED,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for ListUsers {
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/users";

    type SharedCx = SharedContext;
    type HttpRequest = (TypedHeader<BearerToken>, Json<Request>);

    fn request(
        (TypedHeader(token), Json(Request(request))): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(ListUsersRequest {
            auth_token: Some(token),
            ..request
        }
        .into())
    }

    fn response(Response(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for ListUsers {
    const TAG: &'static crate::Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        use crate::user::testing::*;
        [ListUsersResponse {
            cursor: None,
            items: vec![
                User {
                    id: Default::default(),
                    created_at: time::OffsetDateTime::now_utc(),
                    updated_at: time::OffsetDateTime::now_utc(),
                    email: Some(USER_01_EMAIL.into()),
                    username: USER_01_USERNAME.into(),
                    pic_url: Some("https:://example.com/picture.jpg".into()),
                    pub_key: common::utils::encode_hex_multibase(
                        ed25519_dalek::SigningKey::generate(&mut rand::thread_rng())
                            .verifying_key()
                            .to_bytes(),
                    ),
                },
                User {
                    id: Default::default(),
                    created_at: time::OffsetDateTime::now_utc(),
                    updated_at: time::OffsetDateTime::now_utc(),
                    email: Some(USER_02_EMAIL.into()),
                    username: USER_02_USERNAME.into(),
                    pic_url: None,
                    pub_key: common::utils::encode_hex_multibase(
                        ed25519_dalek::SigningKey::generate(&mut rand::thread_rng())
                            .verifying_key()
                            .to_bytes(),
                    ),
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
            ("Access denied", Error::AccessDenied),
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

    use crate::user::list::*;
    use crate::user::testing::*;

    fn fixture_request() -> ListUsersRequest {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "limit": 25,
            "filter": USER_01_USERNAME,
            "sortingField": "username",
            "sortingOrder": "descending"
        })
    }

    common::table_tests! {
        list_users_validate,
        (request, err_field),
        {
            match crate::user::list::validate_request(request) {
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

    list_users_validate! {
        rejects_too_large_limits: (
            ListUsersRequest {
                limit: Some(99999),
                ..fixture_request()
            },
            Some("limit"),
        ),
        rejects_both_cursors_at_once: (
            ListUsersRequest {
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
            ListUsersRequest {
                after_cursor: Some("cursorstr".into()),
                ..fixture_request()
            },
            Some("__all__"),
        ),
    }

    macro_rules! list_users_integ {
        ($(
            $name:ident: {
                auth_token: $auth_token:expr,
                status: $status:expr,
                body: $json_body:expr,
                $(check_json: $check_json:expr,)?
                $(extra_assertions: $extra_fn:expr,)?
            },
        )*) => {
            mod integ {
                use super::*;
                common::integration_table_tests! {
                    $(
                        $name: {
                            uri: "/users",
                            method: "GET",
                            status: $status,
                            router: crate::user::router(),
                            state_fn: crate::utils::testing::state_fn,
                            body: $json_body,
                            $(check_json: $check_json,)?
                            auth_token: $auth_token,
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    list_users_integ! {
        works: {
            auth_token: USER_01_SESSION.into(),
            status: StatusCode::OK,
            body: fixture_request_json().destructure_into_self(serde_json::json!({
                "limit": 2,
                "filter": null,
            })),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn(test_cx);
                    let resp_body_json = response_json.unwrap();
                    assert_eq!(resp_body_json["items"].as_array().unwrap().len(), 2);
                    assert!(resp_body_json["cursor"].as_str().is_some());
                    let app = crate::user::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri("/users")
                                .header(
                                    http::header::AUTHORIZATION,
                                    format!("Bearer {USER_01_SESSION}"),
                                )
                                .header(http::header::CONTENT_TYPE, "application/json")
                                .body(
                                    serde_json::to_vec(
                                        &serde_json::json!({
                                            "afterCursor": resp_body_json["cursor"]
                                                                .as_str()
                                                                .unwrap()
                                        })
                                    ).unwrap().into()
                                )
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
                        2,
                        "{resp_body_json:?}\n{body:?}"
                    );
                    assert!(
                        body["cursor"].is_null(),
                        "{resp_body_json:?}\n{body:?}"
                    );
                })
            },
        },
    }
}
