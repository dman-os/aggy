use crate::interlude::*;

#[derive(Debug, Clone)]
pub struct Reply;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Request {
    #[serde(skip)]
    pub auth_token: Option<BearerToken>,
    pub parent_id: Option<String>,
    #[schema(min_length = 1)]
    #[validate(length(min = 1))]
    pub body: String,
}

pub type Response = Ref<epigram_api::gram::Gram>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("target not found at id: {id}")]
    NotFound { id: String },
    #[error("{self:?}")]
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

#[async_trait::async_trait]
impl crate::AuthenticatedEndpoint for Reply {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    fn authorize_request(&self, request: &Self::Request) -> crate::auth::authorize::Request {
        crate::auth::authorize::Request {
            auth_token: request.auth_token.clone().unwrap(),
            resource: crate::auth::Resource::Replies {
                id: request.parent_id.clone().unwrap(),
            },
            action: crate::auth::Action::Write,
        }
    }

    #[tracing::instrument(skip(cx))]
    async fn handle(
        &self,
        cx: &Self::Cx,
        accessing_user: Uuid,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        validator::Validate::validate(&request).map_err(ValidationErrors::from)?;

        /* match &cx.db {
            crate::Db::Postgres { db_pool } => {},
        }; */
        let (alias, pub_key_str, signing_key) = match &cx.db {
            crate::Db::Pg { db_pool } => {
                let row = sqlx::query!(
                    r#"
SELECT 
    username::TEXT as "username!"
    ,'f' || encode(pub_key, 'hex') as "pub_key!"
    ,pri_key
FROM auth.users
WHERE id = $1::uuid
            "#,
                    &accessing_user
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| match err {
                    sqlx::Error::RowNotFound => Error::AccessDenied,
                    _ => common::internal_err!("db error: {err}"),
                })?;

                (
                    row.username,
                    row.pub_key,
                    ed25519_dalek::SigningKey::from_bytes(
                        &row.pri_key[..].try_into().unwrap_or_log(),
                    ),
                )
            }
        };
        let created_at = OffsetDateTime::now_utc();
        let content = request.body;
        let coty = "text/html".to_string();
        let parent_id = request.parent_id.unwrap();
        let (epigram_id, sig) = epigram_api::utils::hex_id_and_sig_for_gram(
            &signing_key,
            created_at,
            content.as_str(),
            coty.as_str(),
            Some(&parent_id[..]),
        );
        let gram = cx
            .epigram
            .create_gram(epigram_api::gram::create::Request {
                id: epigram_id,
                sig,
                content,
                coty,
                created_at,
                parent_id: Some(parent_id),
                author_alias: Some(alias),
                author_pubkey: pub_key_str,
            })
            .await
            .map_err(|err| {
                use epigram_api::gram::create::Error as Err;
                if err.is::<Err>() {
                    match *err.downcast::<Err>().unwrap_or_log() {
                        Err::ParentNotFound { id } => Error::NotFound { id },
                        err => common::internal_err!(
                            "err trying to create epigram from `epigram_api`: {err}"
                        ),
                    }
                } else {
                    common::internal_err!("err trying to create epigram from `epigram_api`: {err}")
                }
            })?;

        // TODO: email notification, account activation
        Ok(gram)
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            NotFound { .. } => Self::NOT_FOUND,
            AccessDenied => Self::UNAUTHORIZED,
            InvalidInput { .. } => Self::BAD_REQUEST,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for Reply {
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/grams/:id/replies";
    const SUCCESS_CODE: StatusCode = StatusCode::CREATED;

    type SharedCx = SharedContext;
    type HttpRequest = (TypedHeader<BearerToken>, Path<String>, Json<Request>);

    fn request(
        (TypedHeader(auth_token), Path(parent_id), Json(req)): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            auth_token: Some(auth_token),
            parent_id: Some(parent_id),
            ..req
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for Reply {
    const TAG: &'static Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [epigram_api::gram::testing::GRAM_01.clone()]
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Self::Error>> {
        vec![
            (
                "Not Found",
                Error::NotFound {
                    id: epigram_api::gram::testing::GRAM_01_ID.into(),
                },
            ),
            ("Access Denied", Error::AccessDenied),
            (
                "Invalid input",
                Error::InvalidInput {
                    issues: {
                        let mut issues = validator::ValidationErrors::new();
                        issues.add(
                            "email",
                            validator::ValidationError {
                                code: std::borrow::Cow::from("email"),
                                message: None,
                                params: [(
                                    std::borrow::Cow::from("value"),
                                    serde_json::json!("bad.email.com"),
                                )]
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
    use crate::interlude::*;

    use crate::post::testing::*;

    use super::Request;

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "body": r#"Twas bryllyg, and þe slythy toves
Did gyre and gymble in þe wabe:
All mimsy were þe borogoves;
And þe mome raths outgrabe."#,
        })
    }

    common::table_tests! {
        validate,
        (request, err_field),
        {
            match validator::Validate::validate(&request) {
                Ok(()) => {
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

    validate! {
        rejects_empty_bodies: (
            Request {
                body: "".into(),
                ..fixture_request()
            },
            Some("body"),
        ),
    }

    macro_rules! integ {
        ($(
            $name:ident: {
                uri: $uri:expr,
                status: $status:expr,
                auth_token: $auth_token:expr,
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
                            uri: $uri,
                            method: "POST",
                            status: $status,
                            router: crate::post::router(),
                            cx_fn: crate::utils::testing::cx_fn_with_epigram,
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

    integ! {
        works: {
            uri: format!("/grams/{POST_02_EPIGRAM_ID}/replies"),
            status: http::StatusCode::CREATED,
            auth_token: USER_01_SESSION.into(),
            body: fixture_request_json(),
            extra_assertions: &|EAArgs { test_cx, response_json, auth_token, .. }| {
                Box::pin(async move {
                    let cx = state_fn_with_epigram(test_cx);
                    let auth_token = auth_token.unwrap();
                    // let req_body_json = fixture_request_json();
                    let resp_body_json = response_json.unwrap();

                    let app = crate::post::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(format!("/posts/{POST_02_ID}?includeReplies=true"))
                                .body(Default::default())
                                .unwrap_or_log(),
                        )
                        .await
                        .unwrap_or_log();
                    assert_eq!(resp.status(), http::StatusCode::OK);
                    let body = resp.into_body();
                    let body = hyper::body::to_bytes(body).await.unwrap_or_log();
                    let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_log();
                    let replies = body["epigram"]["replies"].as_array().unwrap();
                    assert_eq!(replies.len(), 1);

                    let cx = state_fn_with_epigram(test_cx);
                    let app = crate::post::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("POST")
                                .uri(
                                    format!(
                                        "/grams/{}/replies",
                                        resp_body_json["id"].as_str().unwrap()
                                    )
                                )
                                .header(
                                    axum::http::header::CONTENT_TYPE,
                                    "application/json"
                                )
                                .header(
                                    axum::http::header::AUTHORIZATION,
                                    format!("Bearer {auth_token}")
                                )
                                .body(
                                    serde_json::to_vec(&serde_json::json!({
                                        "body": "Nested comment"
                                    })).unwrap().into()
                                )
                                .unwrap_or_log(),
                        )
                        .await
                        .unwrap_or_log();
                    assert_eq!(resp.status(), http::StatusCode::CREATED);
                })
            },
        },
        // TODO: tests for sanitization
    }
}
