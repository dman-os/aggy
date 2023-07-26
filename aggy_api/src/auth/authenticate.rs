use crate::interlude::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Authenticate;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Request {
    pub identifier: String,
    pub password: String,
}

/// `token` currently appears to be a UUID but don't rely one this as this may
/// change in the future.
#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::FromRow)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Response {
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub token: String,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub expires_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("credentials rejected")]
    CredentialsRejected,
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl Endpoint for Authenticate {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    async fn handle(
        &self,
        cx: &Self::Cx,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let (user_id, pass_hash) = match &cx.db {
            crate::Db::Pg { db_pool } => {
                let result = sqlx::query!(
                    r#"
SELECT user_id, pass_hash
FROM auth.credentials
WHERE user_id = (
    SELECT id
    FROM auth.users
    WHERE email = $1::TEXT::extensions.CITEXT OR username = $1::TEXT::extensions.CITEXT
)
        "#,
                    &request.identifier,
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| match &err {
                    sqlx::Error::RowNotFound => Error::CredentialsRejected,
                    _ => Error::Internal {
                        message: format!("db error: {err}"),
                    },
                })?;
                (result.user_id, result.pass_hash)
            }
        };
        let pass_valid =
            argon2::verify_encoded(&pass_hash[..], request.password.as_bytes()).unwrap();
        if !pass_valid {
            return Err(Error::CredentialsRejected);
        }

        let expires_at =
            time::OffsetDateTime::now_utc().saturating_add(cx.config.auth_token_lifespan);
        let token = uuid::Uuid::new_v4().to_string();
        let out = match &cx.db {
            crate::Db::Pg { db_pool } => {
                sqlx::query_as!(
                    Response,
                    r#"
INSERT INTO auth.sessions (token, user_id, expires_at)
VALUES (
    $1,
    $2,
    $3
) RETURNING
    id AS "session_id!"
    ,token AS "token!"
    ,user_id AS "user_id!"
    ,expires_at AS "expires_at!"
        "#,
                    &token,
                    &user_id,
                    &expires_at
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| Error::Internal {
                    message: format!("db error: {err}"),
                })?
            }
        };
        Ok(out)
    }
}

impl HttpEndpoint for Authenticate {
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/authenticate";

    type SharedCx = SharedContext;
    type HttpRequest = (Json<Request>,);

    fn request((Json(req),): Self::HttpRequest) -> Result<Self::Request, Self::Error> {
        Ok(req)
    }

    fn response(resp: Self::Response) -> axum::response::Response {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for Authenticate {
    const TAG: &'static Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [Self::Response {
            session_id: default(),
            user_id: default(),
            token: "mcpqwen8y3489nc8y2pf".into(),
            expires_at: time::OffsetDateTime::now_utc(),
        }]
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Self::Error>> {
        vec![
            ("Credentials rejected", Error::CredentialsRejected),
            (
                "Internal server error",
                Error::Internal {
                    message: "internal server error".to_string(),
                },
            ),
        ]
    }
}

impl From<&Error> for axum::http::StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            CredentialsRejected { .. } => Self::BAD_REQUEST,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    use crate::user::testing::*;

    use tower::ServiceExt;

    #[tokio::test]
    async fn authenticate_works_with_username() {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
            let app = crate::auth::router().with_state(cx.clone());

            let body_json = serde_json::json!({
                "identifier": USER_01_USERNAME,
                "password": "password",
            });
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("POST")
                        .uri("/authenticate")
                        .header(axum::http::header::CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&body_json).unwrap().into())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::OK);
            let body = resp.into_body();
            let body = hyper::body::to_bytes(body).await.unwrap_or_log();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_log();
            assert!(body["expiresAt"].is_string());
            assert!(body["token"].is_string());
            assert_eq!(USER_01_ID.to_string(), body["userId"].as_str().unwrap());

            let app = crate::user::router().with_state(cx);
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("GET")
                        .uri(format!("/users/{}", body["userId"].as_str().unwrap()))
                        .header(
                            axum::http::header::AUTHORIZATION,
                            format!("Bearer {}", body["token"].as_str().unwrap()),
                        )
                        .body(Default::default())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::OK);
        }
        test_cx.close().await;
    }

    #[tokio::test]
    async fn authenticate_works_with_email() {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
            let app = crate::auth::router().with_state(cx.clone());

            let body_json = serde_json::json!({
                "identifier": USER_01_EMAIL,
                "password": "password",
            });
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("POST")
                        .uri("/authenticate")
                        .header(axum::http::header::CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&body_json).unwrap().into())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::OK);
            let body = resp.into_body();
            let body = hyper::body::to_bytes(body).await.unwrap_or_log();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_log();
            assert!(body["expiresAt"].is_string());
            assert!(body["token"].is_string());
            assert_eq!(USER_01_ID.to_string(), body["userId"].as_str().unwrap());

            let app = crate::user::router().with_state(cx);
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("GET")
                        .uri(format!("/users/{}", body["userId"].as_str().unwrap()))
                        .header(
                            axum::http::header::AUTHORIZATION,
                            format!("Bearer {}", body["token"].as_str().unwrap()),
                        )
                        .body(Default::default())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::OK);
        }
        test_cx.close().await;
    }

    #[tokio::test]
    async fn authenticate_fails_if_username_not_found() {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
            let app = crate::auth::router().with_state(cx);

            let body_json = serde_json::json!({
                "identifier": "golden_eel",
                "password": "password",
            });
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("POST")
                        .uri("/authenticate")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_vec(&body_json).unwrap().into())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
            let body = resp.into_body();
            let body = hyper::body::to_bytes(body).await.unwrap_or_log();
            let body = serde_json::from_slice(&body).unwrap_or_log();
            check_json(
                (
                    "expected",
                    &serde_json::json!({
                        "error": "credentialsRejected"
                    }),
                ),
                ("response", &body),
            );
        }
        test_cx.close().await;
    }

    #[tokio::test]
    async fn authenticate_fails_if_email_not_found() {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
            let app = crate::auth::router().with_state(cx);

            let body_json = serde_json::json!({
                "identifier": "xan@da.man",
                "password": "password",
            });
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("POST")
                        .uri("/authenticate")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_vec(&body_json).unwrap().into())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
            let body = resp.into_body();
            let body = hyper::body::to_bytes(body).await.unwrap_or_log();
            let body = serde_json::from_slice(&body).unwrap_or_log();
            check_json(
                (
                    "expected",
                    &serde_json::json!({
                        "error": "credentialsRejected"
                    }),
                ),
                ("response", &body),
            );
        }
        test_cx.close().await;
    }

    #[tokio::test]
    async fn authenticate_fails_if_password_is_wrong() {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
            let app = crate::auth::router().with_state(cx);

            let body_json = serde_json::json!({
                "identifier": USER_01_EMAIL,
                "password": "apeshit",
            });
            let resp = app
                .oneshot(
                    http::Request::builder()
                        .method("POST")
                        .uri("/authenticate")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_vec(&body_json).unwrap().into())
                        .unwrap_or_log(),
                )
                .await
                .unwrap_or_log();
            assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
            let body = resp.into_body();
            let body = hyper::body::to_bytes(body).await.unwrap_or_log();
            let body = serde_json::from_slice(&body).unwrap_or_log();
            check_json(
                (
                    "expected",
                    &serde_json::json!({
                        "error": "credentialsRejected"
                    }),
                ),
                ("response", &body),
            );
        }
        test_cx.close().await;
    }
}
