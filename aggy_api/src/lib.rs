#![allow(clippy::single_component_path_imports, clippy::let_and_return)]

#[cfg(feature = "dylink")]
#[allow(unused_imports)]
use dylink;

mod interlude {
    pub use crate::{Context, SharedContext};
    pub use axum::{extract::Path, http, response::IntoResponse, Json, TypedHeader};
    pub use common::{
        utils::ValidationErrors, AuthedUid, AuthenticatedEndpoint, Authorize, DocumentedEndpoint,
        Endpoint, EndpointWrapper, ErrorResponse, HttpEndpoint, HttpResponse, Method, Ref,
        StatusCode, Tag,
    };
    pub use deps::*;
    pub type BearerToken = axum::headers::Authorization<axum::headers::authorization::Bearer>;
    pub type DiscardBody = axum::extract::BodyStream;

    #[cfg(test)]
    pub use crate::auth::testing::*;
    #[cfg(test)]
    pub use crate::utils::testing::*;
    #[cfg(test)]
    pub use common::utils::testing::*;
}
use interlude::*;

pub mod auth;
mod macros;
pub mod user;
pub mod utils;

use crate::utils::*;

use utoipa::openapi;

#[derive(Debug)]
pub struct Config {
    pub pass_salt_hash: Vec<u8>,
    pub argon2_conf: argon2::Config<'static>,
    pub auth_token_lifespan: time::Duration,
}

#[derive(Debug)]
pub struct Context {
    pub db_pool: sqlx::postgres::PgPool,
    pub config: Config,
}

pub type SharedContext = std::sync::Arc<Context>;

// shadow_rs::shadow!(build);

pub fn router() -> axum::Router<SharedContext> {
    axum::Router::new()
        .merge(user::router())
        .merge(auth::router())
}

pub struct ApiDoc;
impl utoipa::OpenApi for ApiDoc {
    fn openapi() -> openapi::OpenApi {
        //
        let mut openapi = openapi::OpenApiBuilder::new()
            .info(
                openapi::InfoBuilder::new()
                    .title("aggy_api")
                    // .version(build::PKG_VERSION)
                    .description(Some(format!(
                        r#"{}

Notes:
- Time values are integers despite the `string($date-time)` type shown here.
                        "#,
                        "aggy is an experiment"
                    )))
                    .build(),
            )
            .paths({
                let builder = openapi::path::PathsBuilder::new();
                let builder = user::paths(builder);
                let builder = auth::paths(builder);
                builder.build()
            })
            .components(Some({
                let builder = openapi::ComponentsBuilder::new();
                let builder = builder.schemas_from_iter([
                    <SortingOrder as utoipa::ToSchema>::schema(),
                    <common::utils::ValidationErrors as utoipa::ToSchema>::schema(),
                    <common::utils::ValidationErrorsKind as utoipa::ToSchema>::schema(),
                    <common::utils::ValidationError as utoipa::ToSchema>::schema(),
                ]);
                let builder = user::components(builder);
                let builder = auth::components(builder);
                builder.build()
            }))
            .tags(Some([
                auth::TAG.into(),
                user::TAG.into(),
                common::DEFAULT_TAG.into(),
            ]))
            .build();
        if let Some(components) = openapi.components.as_mut() {
            use utoipa::openapi::security::*;
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(openapi::security::Http::new(
                    openapi::security::HttpAuthScheme::Bearer,
                )),
            )
        }
        openapi
    }
}
