use crate::interlude::*;

use super::Session;

#[derive(Debug, Clone)]
pub struct CreateWebSession;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Request {
    #[serde(skip)]
    pub service_secret: Option<BearerToken>,
    pub ip_addr: std::net::IpAddr,
    pub auth_session_id: Option<Uuid>,
    pub user_agent: String,
}

pub type Response = Ref<Session>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("{self:?}")]
    AccessDenied,
    #[error("user not found at id: {id:?}")]
    AuthSessionNotFound { id: Uuid },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

crate::impl_from_service_auth_err!(Error);

#[async_trait::async_trait]
impl AuthenticatedEndpoint for CreateWebSession {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = ServiceContext;

    fn authorize_request(
        &self,
        request: &Self::Request,
    ) -> crate::auth::authorize_service::Request {
        crate::auth::authorize_service::Request {
            service_secret: request.service_secret.clone().unwrap(),
            resource: crate::auth::Resource::WebSessions,
            action: crate::auth::Action::Write,
        }
    }

    #[tracing::instrument(skip(cx))]
    async fn handle(
        &self,
        cx: &Self::Cx,
        _: (),
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let expires_at = time::OffsetDateTime::now_utc()
            .checked_add(cx.config.web_session_lifespan)
            .unwrap();

        /* match &cx.db {
            crate::Db::Postgres { db_pool } => {},
        }; */
        let out = match &cx.db {
            crate::Db::Pg { db_pool } => {
                /* let result = */
                sqlx::query_as!(
                    Session,
                    r#"
WITH webs as (
    INSERT INTO web.sessions (
        auth_session_id, ip_addr, user_agent, expires_at
    ) VALUES (
        $1::UUID, $2::TEXT::INET, $3, $4
    ) 
    RETURNING *
)
    SELECT
        webs.id as "id!"
    ,   webs.created_at as "created_at!"
    ,   webs.updated_at as "updated_at!"
    ,   webs.expires_at as "expires_at!"
    ,   ip_addr as "ip_addr!: std::net::IpAddr"
    ,   user_agent as "user_agent!"
    ,   auths.expires_at as "token_expires_at?"
    ,   token as "token?"
    ,   user_id as "user_id?"
    FROM (
        webs
            LEFT JOIN
        auth.sessions auths
            ON (webs.auth_session_id = auths.id)
    )
                "#,
                    request.auth_session_id.as_ref(),
                    &request.ip_addr.to_string(),
                    &request.user_agent,
                    &expires_at,
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| {
                    if let sqlx::Error::Database(boxed) = &err {
                        if let Some("sessions_auth_session_id_fkey") = boxed.constraint() {
                            return Error::AuthSessionNotFound {
                                id: request.auth_session_id.unwrap(),
                            };
                        }
                    }
                    panic!("db error: {err}");
                })?
            }
        };
        Ok(out.into())
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            AccessDenied { .. } => Self::UNAUTHORIZED,
            AuthSessionNotFound { .. } => Self::NOT_FOUND,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for CreateWebSession {
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/web/sessions";
    const SUCCESS_CODE: StatusCode = StatusCode::CREATED;

    type SharedCx = SharedServiceContext;
    type HttpRequest = (TypedHeader<BearerToken>, Json<Request>);

    fn request(
        (TypedHeader(token), Json(req)): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            service_secret: Some(token),
            ..req
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for CreateWebSession {
    const TAG: &'static Tag = &crate::web::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [Session {
            id: default(),
            // user_id: Some(default()),
            ip_addr: "127.0.0.1".parse().unwrap(),
            user_agent: "Netscape Nav 119.4".to_string(),
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
            expires_at: time::OffsetDateTime::now_utc(),

            user_id: Some(default()),
            token: Some("joa9wumrilqxu82mdawkl".to_string()),
            token_expires_at: Some(time::OffsetDateTime::now_utc()),
        }]
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Self::Error>> {
        vec![
            ("Access denied", Error::AccessDenied),
            (
                "Auth Session Not Found",
                Error::AuthSessionNotFound { id: default() },
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

    // fn fixture_request() -> Request {
    //     serde_json::from_value(fixture_request_json()).unwrap()
    // }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "ipAddr": "127.0.0.1",
            "userAgent": "Netscape Navigator",
            "authSessionId": USER_01_SESSION,
        })
    }

    macro_rules! integ {
        ($(
            $name:ident: {
                body: $json_body:expr,
                auth_token: $auth_token:expr,
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
                            uri: "/web/sessions",
                            method: "POST",
                            status: $status,
                            router: crate::web::router(),
                            cx_fn: crate::utils::testing::cx_fn_service,
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
            body: fixture_request_json(),
            auth_token: SERVICE_SECRET.into(),
            status: http::StatusCode::CREATED,
            check_json: fixture_request_json().remove_keys_from_obj(&["authSessionId"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn_service(test_cx);
                    let req_body_json = fixture_request_json()
                        .remove_keys_from_obj(&["authSessionId"]);
                    let resp_body_json = response_json.unwrap();

                    let app = crate::web::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(
                                    format!(
                                        "/web/sessions/{}",
                                        resp_body_json["id"].as_str().unwrap()
                                    )
                                )
                                .header(
                                    http::header::AUTHORIZATION,
                                    format!("Bearer {SERVICE_SECRET}"),
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
                        ("expected", &req_body_json),
                        ("response", &body),
                    );
                })
            },
        },
        auth_session_id_is_optional: {
            body: fixture_request_json().remove_keys_from_obj(&["authSessionId"]),
            auth_token: SERVICE_SECRET.into(),
            status: http::StatusCode::CREATED,
            check_json: fixture_request_json().remove_keys_from_obj(&["authSessionId"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn_service(test_cx);
                    let req_body_json = fixture_request_json()
                        .remove_keys_from_obj(&["authSessionId"]);
                    let resp_body_json = response_json.unwrap();

                    let app = crate::web::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(
                                    format!(
                                        "/web/sessions/{}",
                                        resp_body_json["id"].as_str().unwrap()
                                    )
                                )
                                .header(
                                    http::header::AUTHORIZATION,
                                    format!("Bearer {SERVICE_SECRET}"),
                                )
                                .body(Default::default())
                                .unwrap_or_log(),
                        )
                        .await
                        .unwrap_or_log();

                    let (head, body) = resp.into_parts();
                    let body = hyper::body::to_bytes(body).await.unwrap_or_log();
                    let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_log();
                    assert_eq!(head.status, StatusCode::OK, "{head:?} {body:?}");
                    check_json(
                        ("expected", &req_body_json),
                        ("response", &body),
                    );
                })
            },
        },
        fails_if_auth_session_not_found: {
            body: fixture_request_json().destructure_into_self(
                serde_json::json!({ "authSessionId": Uuid::new_v4() })
            ),
            auth_token: SERVICE_SECRET.into(),
            status: http::StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "authSessionNotFound"
            }),
        },
    }
}
