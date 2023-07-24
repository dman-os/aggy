use crate::interlude::*;

use super::User;

#[derive(Clone, Copy, Debug)]
pub struct GetUser;

#[derive(Debug)]
pub struct Request {
    pub auth_token: BearerToken,
    pub id: Uuid,
}

pub type Response = Ref<super::User>;

#[derive(Debug, thiserror::Error, serde::Serialize, utoipa::ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    #[error("user not found at id: {id:?}")]
    NotFound { id: Uuid },
    #[error("{self:?}")]
    AccessDenied,
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

crate::impl_from_auth_err!(Error);

#[async_trait::async_trait]
impl crate::AuthenticatedEndpoint for GetUser {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    fn authorize_request(&self, request: &Self::Request) -> crate::auth::authorize::Request {
        crate::auth::authorize::Request {
            auth_token: request.auth_token.clone(),
            resource: crate::auth::Resource::User { id: request.id },
            action: crate::auth::Action::Read,
        }
    }

    #[tracing::instrument(skip(cx))]
    async fn handle(
        &self,
        cx: &Self::Cx,
        _accessing_user: Uuid,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let id = request.id;

        match &cx.db {
            crate::Db::Pg { db_pool } => sqlx::query_as!(
                User,
                r#"
SELECT 
    id
    ,created_at
    ,updated_at
    ,email::TEXT as "email?"
    ,username::TEXT as "username!"
    ,'f' || encode(pub_key, 'hex') as "pub_key!"
    ,pic_url
FROM auth.users
WHERE id = $1::uuid
            "#,
                &id
            )
            .fetch_one(db_pool)
            .await
            .map(|val| val.into())
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => Error::NotFound { id },
                _ => Error::Internal {
                    message: format!("db error: {err}"),
                },
            }),
        }
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            NotFound { .. } => Self::NOT_FOUND,
            AccessDenied => Self::UNAUTHORIZED,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for GetUser {
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/users/:id";

    type SharedCx = SharedContext;
    type HttpRequest = (TypedHeader<BearerToken>, Path<Uuid>, DiscardBody);

    fn request(
        (TypedHeader(token), Path(id), _): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(self::Request {
            auth_token: token,
            id,
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for GetUser {
    const TAG: &'static crate::Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        use crate::user::testing::*;
        [User {
            id: Default::default(),
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
            email: Some(USER_01_EMAIL.into()),
            username: USER_01_USERNAME.into(),
            pic_url: Some("https:://example.com/picture.jpg".into()),
            pub_key: crate::utils::encode_hex_multibase(
                ed25519_dalek::SigningKey::generate(&mut rand::thread_rng())
                    .verifying_key()
                    .to_bytes(),
            ),
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
                "Not found",
                Error::NotFound {
                    id: Default::default(),
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

    use crate::user::testing::*;

    macro_rules! get_user_integ {
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
                            router: crate::user::router(),
                            state_fn: crate::utils::testing::state_fn,
                            $(check_json: $check_json,)?
                            auth_token: $auth_token,
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    get_user_integ! {
        works: {
            uri: format!("/users/{USER_01_ID}"),
            auth_token: USER_01_SESSION.into(),
            status: StatusCode::OK,
            check_json: serde_json::json!({
                "id": USER_01_ID,
                "username": USER_01_USERNAME,
                "email": USER_01_EMAIL,
            }),
        },
        fails_if_not_found: {
            uri: format!("/users/{}", Uuid::new_v4()),
            auth_token: USER_01_SESSION.into(), // FIXME: use super user session
            status: StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound",
            }),
        },
    }
}
