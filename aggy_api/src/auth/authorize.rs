//! Check if the auth policy allows the provded [`User`] is allowed
//! to perform the provided [`Action`] on the provided [`Resource`]

use crate::interlude::*;

use crate::auth::{Action, Resource};

#[derive(Debug)]
pub struct Request {
    pub auth_token: BearerToken,
    pub resource: Resource,
    pub action: Action,
}

#[derive(Debug, thiserror::Error, serde::Serialize, utoipa::ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid token")]
    InvalidToken,
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl common::Authorize for crate::Context {
    type Info = uuid::Uuid;
    type Request = Request;
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn authorize(&self, request: Self::Request) -> Result<Self::Info, Self::Error> {
        // TODO: roles support
        // TODO: cache db access

        let session = match &self.db {
            crate::Db::Pg { db_pool } => sqlx::query_as!(
                super::Session,
                r#"
SELECT * 
FROM auth.sessions
WHERE token = $1
            "#,
                &request.auth_token.token()
            )
            .fetch_one(db_pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => Error::InvalidToken,
                _ => Error::Internal {
                    message: format!("{err}"),
                },
            })?,
        };
        if session.expires_at < time::OffsetDateTime::now_utc() {
            return Err(Error::InvalidToken);
        }
        Ok(session.user_id)
    }
}

#[cfg(test)]
mod tests {
    // use deps::*;

    use crate::interlude::*;

    use crate::{auth::*, user::testing::*, Endpoint};

    common::table_tests! {
        authorize_policy tokio,
        (username, id, resource_actions),
        {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
                let res = authenticate::Authenticate.handle(&cx, authenticate::Request{
                    identifier: username.to_string(),
                    password: "password".into()
                }).await.unwrap_or_log();
                for (resource, action) in resource_actions {
                    let user_id = cx.authorize(authorize::Request {
                        auth_token: BearerToken::bearer(&res.token[..]).unwrap_or_log(),
                        resource,
                        action
                    }).await.unwrap_or_log();
                    assert_eq!(id, user_id);
                }
            }
            test_cx.close().await;
        },
    }

    authorize_policy! {
        allows_any_action_on_own_account: (
            USER_01_USERNAME,
            USER_01_ID,
            {
                [
                    Resource::User { id: USER_01_ID }
                ]
                .into_iter()
                .flat_map(|res| {
                    [
                        Action::Read,
                        Action::Write,
                        Action::Delete
                    ]
                    .into_iter()
                    .map(move |act| (res.clone(), act))
                }).collect::<Vec<_>>()
            }
        ),
    }
}
