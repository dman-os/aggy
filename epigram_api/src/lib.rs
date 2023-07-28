#![allow(clippy::single_component_path_imports, clippy::let_and_return)]

#[cfg(feature = "dylink")]
#[allow(unused_imports)]
use dylink;

mod interlude {
    pub use deps::*;

    pub use crate::{Context, ServiceContext, SharedContext, SharedServiceContext};

    pub use axum::{extract::Path, http, response::IntoResponse, Json, TypedHeader};
    pub use serde::{Deserialize, Serialize};
    pub use time::format_description::well_known::Iso8601;
    pub use utoipa::ToSchema;
    pub use uuid::Uuid;
    pub use validator::Validate;

    pub use common::utils::default;
    pub use common::{
        utils::ValidationErrors, AuthedUid, AuthenticatedEndpoint, Authorize, DocumentedEndpoint,
        Endpoint, EndpointWrapper, ErrorResponse, HttpEndpoint, HttpResponse, Method, Ref,
        StatusCode, Tag,
    };

    pub type BearerToken = axum::headers::Authorization<axum::headers::authorization::Bearer>;
    pub type DiscardBody = axum::extract::BodyStream;

    #[cfg(test)]
    pub use crate::utils::testing::*;
    #[cfg(test)]
    pub use common::utils::testing::*;
}
use interlude::*;

pub mod gram;
mod macros;
mod utils;

use crate::utils::*;

use utoipa::openapi;

#[derive(Debug)]
pub struct Config {
    pub pass_salt_hash: Vec<u8>,
    pub argon2_conf: argon2::Config<'static>,
    pub auth_token_lifespan: time::Duration,
    pub web_session_lifespan: time::Duration,
    pub service_secret: String,
}

#[derive(Debug)]
pub struct Context {
    pub config: Config,
    pub db: Db,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Db {
    Pg { db_pool: sqlx::postgres::PgPool },
}

pub type SharedContext = std::sync::Arc<Context>;

#[derive(educe::Educe, Clone)]
#[educe(Deref, DerefMut)]
pub struct ServiceContext(pub SharedContext);

#[derive(educe::Educe, Clone)]
#[educe(Deref, DerefMut)]
pub struct SharedServiceContext(pub ServiceContext);

impl axum::extract::FromRef<SharedContext> for SharedServiceContext {
    fn from_ref(input: &SharedContext) -> Self {
        Self(ServiceContext(input.clone()))
    }
}
// shadow_rs::shadow!(build);

pub fn router(state: SharedContext) -> axum::Router {
    axum::Router::new().with_state(state.clone())
    // .merge(web::router().with_state(SharedServiceContext(ServiceContext(state))))
}

pub struct ApiDoc;
impl utoipa::OpenApi for ApiDoc {
    fn openapi() -> openapi::OpenApi {
        //
        let mut openapi = openapi::OpenApiBuilder::new()
            .info(
                openapi::InfoBuilder::new()
                    .title("epigram_api")
                    // .version(build::PKG_VERSION)
                    .description(Some(format!(
                        r#"{}
                        "#,
                        "epigram is an experiment"
                    )))
                    .build(),
            )
            .paths({
                let builder = openapi::path::PathsBuilder::new();
                // let builder = user::paths(builder, "/epigram"); //FIXME: make this dyamic
                builder.build()
            })
            .components(Some({
                let builder = openapi::ComponentsBuilder::new();
                let builder = builder
                    .schema(
                        "std.net.IpAddr",
                        openapi::ObjectBuilder::new()
                            .schema_type(openapi::SchemaType::String)
                            .format(Some(openapi::SchemaFormat::Custom("ipaddr".into()))),
                    )
                    .schemas_from_iter([
                        <SortingOrder as utoipa::ToSchema>::schema(),
                        <common::utils::ValidationErrors as utoipa::ToSchema>::schema(),
                        <common::utils::ValidationErrorsKind as utoipa::ToSchema>::schema(),
                        <common::utils::ValidationError as utoipa::ToSchema>::schema(),
                    ]);
                // let builder = user::components(builder);
                builder.build()
            }))
            .tags(Some([
                // auth::TAG.into(),
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
