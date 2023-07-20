use crate::interlude::*;

use super::{Action, Resource};

#[derive(Debug)]
pub struct Request {
    pub service_secret: BearerToken,
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
impl common::Authorize for ServiceContext {
    type Info = ();
    type Request = Request;
    type Error = Error;

    #[tracing::instrument(skip(self))]
    async fn authorize(&self, request: Self::Request) -> Result<Self::Info, Self::Error> {
        let (Resource::WebSessions | Resource::WebSession { .. }) = request.resource else {
            return Err(Error::Unauthorized)
        };
        if self.0.config.service_secret != request.service_secret.token() {
            return Err(Error::InvalidToken);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use deps::*;

    use crate::interlude::*;

    use crate::{auth::*, user::testing::*};

    common::table_tests! {
        authorize_policy tokio,
        (service_secret, resource_actions),
        {
        let test_cx = TestContext::new(common::function!()).await;
        {
            let cx = state_fn(&test_cx);
            let cx = SharedServiceContext(ServiceContext(cx));
            for (resource, action) in resource_actions {
                cx.authorize(authorize_service::Request {
                    service_secret: BearerToken::bearer(&service_secret[..]).unwrap_or_log(),
                    resource,
                    action
                }).await.unwrap_or_log();
            }
            }
            test_cx.close().await;
        },
    }

    authorize_policy! {
        allows_any_action_on_web_sessions: (
            SERVICE_SECRET,
            {
                [
                    Resource::WebSession { id: USER_01_ID },
                    Resource::WebSessions
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
