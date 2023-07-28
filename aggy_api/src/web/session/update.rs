use crate::interlude::*;

use super::Session;

#[derive(Debug, Clone)]
pub struct UpdateWebSession;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Request {
    #[serde(skip)]
    pub service_secret: Option<BearerToken>,
    #[serde(skip)]
    pub session_id: Option<uuid::Uuid>,
    pub auth_session_id: Option<Uuid>,
}

pub type Response = Ref<Session>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("{self:?}")]
    AccessDenied,
    #[error("session not found at id: {id:?}")]
    NotFound { id: uuid::Uuid },
    #[error("auth session not found at id: {id:?}")]
    AuthSessionNotFound { id: Uuid },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

crate::impl_from_service_auth_err!(Error);

#[async_trait::async_trait]
impl AuthenticatedEndpoint for UpdateWebSession {
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
            resource: crate::auth::Resource::WebSession {
                id: request.session_id.unwrap(),
            },
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
        let out = match &cx.db {
            crate::Db::Pg { db_pool } => sqlx::query_as!(
                Session,
                r#"
WITH webs as (
    UPDATE web.sessions 
    SET 
        auth_session_id = COALESCE($2, auth_session_id)
    WHERE id = $1 
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
    ,   token
    ,   user_id
    FROM (
        webs
            LEFT JOIN
        auth.sessions auths
            ON (webs.auth_session_id = auths.id)
    )
                "#,
                &request.session_id.unwrap(),
                request.auth_session_id.as_ref(),
            )
            .fetch_one(db_pool)
            .await
            .map_err(|err| match &err {
                sqlx::Error::RowNotFound => Error::NotFound {
                    id: request.session_id.unwrap(),
                },
                sqlx::Error::Database(boxed) if boxed.constraint().is_some() => {
                    match boxed.constraint().unwrap() {
                        "sessions_auth_session_id_fkey" => Error::AuthSessionNotFound {
                            id: request.auth_session_id.unwrap(),
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
        Ok(out.into())
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            AccessDenied { .. } => Self::UNAUTHORIZED,
            NotFound { .. } | AuthSessionNotFound { .. } => Self::NOT_FOUND,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for UpdateWebSession {
    type SharedCx = SharedServiceContext;

    const METHOD: Method = Method::Patch;
    const PATH: &'static str = "/web/sessions/:id";

    type HttpRequest = (TypedHeader<BearerToken>, Path<Uuid>, Json<Request>);

    fn request(
        (TypedHeader(token), Path(session_id), Json(req)): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            service_secret: Some(token),
            session_id: Some(session_id),
            ..req
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for UpdateWebSession {
    const TAG: &'static Tag = &crate::web::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [Session {
            id: default(),
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
            ("Not Found", Error::NotFound { id: default() }),
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

    use crate::user::testing::*;
    use crate::web::session::testing::*;

    // fn fixture_request() -> Request {
    //     serde_json::from_value(fixture_request_json()).unwrap()
    // }

    fn fixture_request_json() -> serde_json::Value {
        serde_json::json!({
            "authSessionId": USER_04_SESSION,
        })
    }

    macro_rules! integ {
        ($(
            $name:ident: {
                uri: $uri:expr,
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
                            uri: $uri,
                            method: "PATCH",
                            status: $status,
                            router: crate::web::router(),
                            state_fn: crate::utils::testing::state_fn_service,
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
            uri: format!("/web/sessions/{USER_01_WEB_SESSION}"),
            body: fixture_request_json(),
            auth_token: SERVICE_SECRET.into(),
            status: http::StatusCode::OK,
            check_json: serde_json::json!({ "userId": USER_04_ID }),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn_service(test_cx);
                    let req_body_json = serde_json::json!({ "userId": USER_04_ID });
                    let resp_body_json = response_json.unwrap();

                    let app = crate::web::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(
                                    format!("/web/sessions/{USER_01_WEB_SESSION}",)
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
                    check_json(
                        ("expected", &req_body_json),
                        ("response", &body),
                    );
                })
            },
        },
        fails_if_not_found: {
            uri: format!("/web/sessions/{}", Uuid::new_v4()),
            body: fixture_request_json(),
            auth_token: SERVICE_SECRET.into(),
            status: http::StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound"
            }),
        },
        fails_if_user_not_found: {
            uri: format!("/web/sessions/{USER_01_WEB_SESSION}"),
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
