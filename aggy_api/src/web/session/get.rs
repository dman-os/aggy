use crate::interlude::*;

use super::Session;

#[derive(Debug, Clone)]
pub struct GetWebSession;

#[derive(Debug)]
pub struct Request {
    pub service_secret: Option<BearerToken>,
    pub id: Uuid,
}

pub type Response = Ref<Session>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("{self:?}")]
    AccessDenied,
    #[error("session not found at id: {id:?}")]
    NotFound { id: Uuid },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

crate::impl_from_service_auth_err!(Error);

#[async_trait::async_trait]
impl AuthenticatedEndpoint for GetWebSession {
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
            resource: crate::auth::Resource::WebSession { id: request.id },
            action: crate::auth::Action::Read,
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
            // FIXME: enable ipnetwork in sqlx
            crate::Db::Pg { db_pool } => {
                /* let result = */
                sqlx::query_as!(
                    Session,
                    r#"
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
    web.sessions webs
        LEFT JOIN
    auth.sessions auths
        ON (webs.auth_session_id = auths.id)
)
WHERE webs.id = $1
    ;
                "#,
                    &request.id
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| match &err {
                    sqlx::Error::RowNotFound => Error::NotFound { id: request.id },
                    _ => common::internal_err!("db error: {err}"),
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
            NotFound { .. } => Self::NOT_FOUND,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for GetWebSession {
    type SharedCx = SharedServiceContext;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/web/sessions/:id";

    type HttpRequest = (TypedHeader<BearerToken>, Path<Uuid>, DiscardBody);

    fn request(
        (TypedHeader(token), Path(id), _): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            service_secret: Some(token),
            id,
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for GetWebSession {
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

    macro_rules! integ {
        ($(
            $name:ident: {
                uri: $uri:expr,
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
                            method: "GET",
                            status: $status,
                            router: crate::web::router(),
                            cx_fn: crate::utils::testing::cx_fn_service,
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
            auth_token: SERVICE_SECRET.into(),
            status: http::StatusCode::OK,
            check_json: serde_json::json!({
                "id": USER_01_WEB_SESSION,
                "ipAddr": "127.0.0.1",
                "userAgent": "ViolaWWW",
                "userId": USER_01_ID,
            }),
        },
        fails_if_not_found: {
            uri: format!("/web/sessions/{}", Uuid::new_v4()),
            auth_token: SERVICE_SECRET.into(),
            status: StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound",
            }),
        },
        fails_if_bad_service_secret: {
            uri: format!("/web/sessions/{USER_01_WEB_SESSION}"),
            auth_token: "TELL ME!".into(),
            status: StatusCode::UNAUTHORIZED,
            check_json: serde_json::json!({
                "error": "accessDenied",
            }),
        },
    }
}
