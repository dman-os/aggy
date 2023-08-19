use crate::interlude::*;

use super::Post;

#[derive(Debug, Clone)]
pub struct CreatePost;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
#[validate(schema(function = "validate_req"))]
pub struct Request {
    #[serde(skip)]
    pub auth_token: Option<BearerToken>,
    #[schema(min_length = 1, max_length = 80)]
    #[validate(length(min = 1, max = 80))]
    pub title: String,
    #[validate(url)]
    pub url: Option<String>,
    #[schema(min_length = 1)]
    #[validate(length(min = 1))]
    pub body: Option<String>,
}

fn validate_req(req: &Request) -> Result<(), validator::ValidationError> {
    if req.url.is_none() && req.body.is_none() {
        return Err(validator::ValidationError {
            code: Cow::from("both_url_and_body_missing"),
            message: Some(Cow::from("Either url or body must be present")),
            params: default(),
        });
    }
    Ok(())
    // todo!("check if either url or body are present; sanitize html")
}

pub type Response = Ref<super::Post>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
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
impl crate::AuthenticatedEndpoint for CreatePost {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    fn authorize_request(&self, request: &Self::Request) -> crate::auth::authorize::Request {
        crate::auth::authorize::Request {
            auth_token: request.auth_token.clone().unwrap(),
            resource: crate::auth::Resource::Posts,
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

        let Request {
            title, url, body, ..
        } = request;
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
        // FIXME: replace with v7
        let post_id = Uuid::new_v4();
        let created_at = OffsetDateTime::now_utc();
        let content = match (url.as_ref(), body.as_ref()) {
            (Some(url), Some(body)) => format!(
                r#"<a href="{url}">{title}</a>

<p>{body}</p>"#
            ),
            (Some(url), None) => format!(r#"<a href="{url}">{title}</a>"#),
            // FIXME: parameterize aggydomain
            (None, Some(body)) => format!(
                r#"<a href="https://aggy.news/p/{post_id}">{title}</a>

<p>{body}</p>"#
            ),
            (None, None) => format!(r#"<a href="https://aggy.news/p/{post_id}">{title}</a>"#),
        };
        let coty = "text/html".to_string();
        let (epigram_id, sig) = epigram_api::utils::hex_id_and_sig_for_gram(
            &signing_key,
            created_at,
            content.as_str(),
            coty.as_str(),
            None,
        );
        let gram = cx
            .epigram
            .create_gram(epigram_api::gram::create::Request {
                id: epigram_id,
                sig,
                content,
                coty,
                created_at,
                parent_id: None,
                author_alias: Some(alias),
                author_pubkey: pub_key_str,
            })
            .await
            .map_err(|err| {
                common::internal_err!("err trying to create epigram from `epigram_api`: {err}")
            })?;
        let epigram_id = common::utils::decode_hex_multibase(gram.id.as_str()).unwrap_or_log();
        let item = match &cx.db {
            crate::Db::Pg { db_pool } => {
                let row = sqlx::query!(
                    r#"
WITH post AS (
    INSERT INTO posts.posts (
        id
        ,created_at
        ,updated_at
        ,author_id
        ,epigram_id
        ,title
        ,url
        ,body
    )
    VALUES (
        $1::UUID, $2 ,$3 ,$4 ,$5 ,$6 ,$7 ,$8
    ) RETURNING *
) SELECT 
    p.created_at as "created_at!"
    ,p.updated_at as "updated_at!"
    ,p.id as "id"
    ,p.title as "title"
    ,p.url as "url"
    ,p.body as "body"
    ,util.multibase_encode_hex(p.epigram_id) as "epigram_id!"
    ,util.multibase_encode_hex(u.pub_key) as "author_pub_key!"
    ,u.username::TEXT as "author_username!"
    ,u.pic_url as "author_pic_url"
FROM 
    post as p
        LEFT JOIN
    auth.users as u
        ON (p.author_id = u.id)
                                "#,
                    &post_id,
                    &created_at,
                    &created_at,
                    &accessing_user,
                    &epigram_id,
                    &title,
                    url.as_ref(),
                    body.as_ref(),
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| common::internal_err!("db error: {err}"))?;
                Post {
                    id: row.id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    epigram_id: row.epigram_id,
                    title: row.title,
                    url: row.url,
                    body: row.body,
                    author_username: row.author_username,
                    author_pub_key: row.author_pub_key,
                    author_pic_url: row.author_pic_url,
                    epigram: Some(gram.0),
                }
            }
        };

        // TODO: email notification, account activation
        Ok(item.into())
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            AccessDenied => Self::UNAUTHORIZED,
            InvalidInput { .. } => Self::BAD_REQUEST,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for CreatePost {
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/posts";
    const SUCCESS_CODE: StatusCode = StatusCode::CREATED;

    type SharedCx = SharedContext;
    type HttpRequest = (TypedHeader<BearerToken>, Json<Request>);

    fn request(
        (TypedHeader(auth_token), Json(req)): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            auth_token: Some(auth_token),
            ..req
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for CreatePost {
    const TAG: &'static Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [Post {
            id: default(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            epigram_id: "f26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61".into(),
            body: Some("Please sign in to see this xeet.".into()),
            title: "Earth 2 reported to begin operations next circumsolar year".into(),
            url: Some("ipns://𝕏.com/stella_oort/48723494675897134423".into()),
            author_username: "tazental".into(),
            author_pic_url: None,
            author_pub_key: "f196b70071ff6d9c6480677814ac78d2d1478a05a46c60d1dcd7afd21befb0b89"
                .into(),
            epigram: None,
        }]
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Self::Error>> {
        vec![
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

    use super::Request;

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "title": "Goverments are replacing the death penalty with hellvisor faiclities",
            "url": "https://example.com/",
            "body": "Whatdyya prefer ladies, death or a million years of digital Dante simulation?",
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
        rejects_empty_titles: (
            Request {
                title: "".into(),
                ..fixture_request()
            },
            Some("title"),
        ),
        rejects_too_long_titles: (
            Request {
                title: "I'm going to beat it like the congo of god \"big daddy\" almighty himself, the big gong for judgment day".into(),
                ..fixture_request()
            },
            Some("title"),
        ),
        rejects_if_both_url_and_title_is_missing: (
            Request {
                url: None,
                body: None,
                ..fixture_request()
            },
            Some("__all__"),
        ),
        rejects_invalid_urls: (
            Request {
                url: Some("invalid".into()),
                ..fixture_request()
            },
            Some("url"),
        ),
        rejects_empty_bodies: (
            Request {
                body: Some("".into()),
                ..fixture_request()
            },
            Some("body"),
        ),
    }

    macro_rules! integ {
        ($(
            $name:ident: {
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
                            uri: "/posts",
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
            status: http::StatusCode::CREATED,
            auth_token: USER_01_SESSION.into(),
            body: fixture_request_json(),
            check_json: fixture_request_json(),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn(test_cx);
                    let req_body_json = fixture_request_json();
                    let resp_body_json = response_json.unwrap();

                    let app = crate::post::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(format!("/posts/{}", resp_body_json["id"].as_str().unwrap()))
                                .body(Default::default())
                                .unwrap_or_log(),
                        )
                        .await
                        .unwrap_or_log();
                    assert_eq!(resp.status(), http::StatusCode::OK);
                    let body = resp.into_body();
                    let body = hyper::body::to_bytes(body).await.unwrap_or_log();
                    let body = serde_json::from_slice(&body).unwrap_or_log();
                    tracing::info!(?body, "test");
                    check_json(
                        ("expected", &req_body_json.remove_keys_from_obj(&["password"])),
                        ("response", &body),
                    );
                })
            },
        },
        url_is_optional: {
            status: http::StatusCode::CREATED,
            auth_token: USER_01_SESSION.into(),
            body: fixture_request_json().remove_keys_from_obj(&["url"]),
            check_json: fixture_request_json().remove_keys_from_obj(&["url"]),
        },
        body_is_optional: {
            status: http::StatusCode::CREATED,
            auth_token: USER_01_SESSION.into(),
            body: fixture_request_json().remove_keys_from_obj(&["body"]),
            check_json: fixture_request_json().remove_keys_from_obj(&["body"]),
        },
        // TODO: tests for sanitization
    }
}
