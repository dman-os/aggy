use crate::interlude::*;

#[test]
#[ignore]
fn gen_pub_pri_key() {
    let pri_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let pub_key = pri_key.verifying_key();
    println!(
        "pri_key: {}",
        common::utils::encode_hex_multibase(pri_key.to_bytes())
    );
    println!(
        "pub_key: {}",
        common::utils::encode_hex_multibase(pub_key.to_bytes())
    );
}

#[derive(Debug, Clone)]
pub struct CreateUser;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Request {
    #[schema(
        min_length = 5,
        max_length = 32,
        pattern = "^[a-zA-Z0-9]+([_-]?[a-zA-Z0-9])*$"
    )]
    #[validate(length(min = 5, max = 25), regex(path = "crate::user::USERNAME_REGEX"))]
    pub username: String,
    /// Must be a valid email string
    #[validate(email)]
    pub email: Option<String>,
    #[schema(min_length = 8, max_length = 1024)]
    #[validate(length(min = 8, max = 1024))]
    pub password: String,
}

pub type Response = Ref<super::User>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("username occupied: {username:?}")]
    UsernameOccupied { username: String },
    #[error("email occupied: {email:?}")]
    EmailOccupied { email: String },
    #[error("invalid input: {issues:?}")]
    InvalidInput {
        #[from]
        issues: ValidationErrors,
    },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl Endpoint for CreateUser {
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
        validator::Validate::validate(&request).map_err(ValidationErrors::from)?;
        let pass_hash = argon2::hash_encoded(
            request.password.as_bytes(),
            &cx.config.pass_salt_hash,
            &cx.config.argon2_conf,
        )
        .unwrap_or_log();
        let pri_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
        let pub_key = pri_key.verifying_key();
        let pri_key = pri_key.to_bytes();
        let pub_key = pub_key.to_bytes();

        /* match &cx.db {
            crate::Db::Postgres { db_pool } => {},
        }; */
        let user = match &cx.db {
            crate::Db::Pg { db_pool } => sqlx::query_as!(
                super::User,
                r#"
SELECT
    id as "id!"
    ,created_at as "created_at!"
    ,updated_at as "updated_at!"
    ,email::TEXT as "email?"
    ,username::TEXT as "username!"
    ,'f' || encode(pub_key, 'hex') as "pub_key!"
    ,pic_url
FROM auth.create_user($1, $2, $3, $4, $5)
                "#,
                &request.username,
                request.email.as_ref(),
                &pass_hash,
                &pub_key,
                &pri_key,
            )
            .fetch_one(db_pool)
            .await
            .map_err(|err| match &err {
                sqlx::Error::Database(boxed) if boxed.constraint().is_some() => {
                    match boxed.constraint().unwrap() {
                        "users_username_key" => Error::UsernameOccupied {
                            username: request.username,
                        },
                        "users_email_key" => Error::EmailOccupied {
                            email: request.email.unwrap(),
                        },
                        _ => Error::Internal {
                            message: format!("db error: {err}"),
                        },
                    }
                }
                _ => Error::Internal {
                    message: format!("db error: {err}"),
                },
            })?,
        };
        // TODO: email notification, account activation
        Ok(user.into())
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            UsernameOccupied { .. } | EmailOccupied { .. } | InvalidInput { .. } => {
                Self::BAD_REQUEST
            }
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for CreateUser {
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/users";
    const SUCCESS_CODE: StatusCode = StatusCode::CREATED;

    type SharedCx = SharedContext;
    type HttpRequest = (Json<Request>,);

    fn request((Json(req),): Self::HttpRequest) -> Result<Self::Request, Self::Error> {
        Ok(req)
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for CreateUser {
    const TAG: &'static Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        use crate::user::testing::*;
        [super::User {
            id: Default::default(),
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
            email: Some(USER_01_EMAIL.into()),
            username: USER_01_USERNAME.into(),
            pub_key: common::utils::encode_hex_multibase(
                ed25519_dalek::SigningKey::generate(&mut rand::thread_rng())
                    .verifying_key()
                    .to_bytes(),
            ),
            pic_url: Some("https:://example.com/picture.jpg".into()),
        }]
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Self::Error>> {
        use crate::user::testing::*;
        vec![
            (
                "Username occupied",
                Error::UsernameOccupied {
                    username: USER_01_USERNAME.into(),
                },
            ),
            (
                "Email occupied",
                Error::EmailOccupied {
                    email: USER_01_EMAIL.into(),
                },
            ),
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

    use crate::user::testing::*;
    use crate::{auth::*, Endpoint};

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "username": "whish_box12",
            "email": "multis@cream.mux",
            "password": "lovebite",
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
        rejects_too_long_usernames: (
            Request {
                username: "shrt".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_too_short_usernames: (
            Request {
                username: "man-the-manly-man-ende-man-be-man-eske-man123".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_usernames_that_ends_with_dashes: (
            Request {
                username: "wrenz-".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_usernames_that_start_with_dashes: (
            Request {
                username: "-wrenz".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_usernames_that_ends_with_underscore: (
            Request {
                username: "belle_".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_usernames_that_start_with_underscore: (
            Request {
                username: "_belle".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_usernames_with_white_space: (
            Request {
                username: "daddy yo".into(),
                ..fixture_request()
            },
            Some("username"),
        ),
        rejects_too_short_passwords: (
            Request {
                password: "short".into(),
                ..fixture_request()
            },
            Some("password"),
        ),
        rejects_invalid_emails: (
            Request {
                email: Some("invalid".into()),
                ..fixture_request()
            },
            Some("email"),
        ),
    }

    macro_rules! integ {
        ($(
            $name:ident: {
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
                            method: "POST",
                            status: $status,
                            router: crate::user::router(),
                            cx_fn: crate::utils::testing::cx_fn,
                            body: $json_body,
                            $(check_json: $check_json,)?
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
            body: fixture_request_json(),
            check_json: fixture_request_json().remove_keys_from_obj(&["password"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn(test_cx);
                    let req_body_json = fixture_request_json();
                    let resp_body_json = response_json.unwrap();
                    // TODO: use super user token
                    let token = authenticate::Authenticate.handle(&cx,authenticate::Request{
                        identifier: req_body_json["username"].as_str().unwrap().into(),
                        password: req_body_json["password"].as_str().unwrap().into()
                    }).await.unwrap_or_log().token;

                    let app = crate::user::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(format!("/users/{}", resp_body_json["id"].as_str().unwrap()))
                                .header(
                                    http::header::AUTHORIZATION,
                                    format!("Bearer {token}"),
                                )
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
        email_is_optional: {
            status: http::StatusCode::CREATED,
            body: fixture_request_json().remove_keys_from_obj(&["email"]),
            check_json: fixture_request_json().remove_keys_from_obj(&["password", "email"]),
        },
        fails_if_username_occupied: {
            status: http::StatusCode::BAD_REQUEST,
            body: fixture_request_json().destructure_into_self(
                serde_json::json!({ "username": USER_01_USERNAME })
            ),
            check_json: serde_json::json!({
                "error": "usernameOccupied"
            }),
        },
        /*
        // FIXME:
        fails_if_email_occupied: {
            status: http::StatusCode::BAD_REQUEST,
            body: fixture_request_json().destructure_into_self(
                serde_json::json!({ "email": USER_01_EMAIL })
            ),
            check_json: serde_json::json!({
                "error": "emailOccupied"
            }),
        },*/
    }
}
