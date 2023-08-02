#![allow(clippy::single_component_path_imports, clippy::let_and_return)]

#[cfg(feature = "dylink")]
#[allow(unused_imports)]
use dylink;

mod interlude {
    pub use deps::*;

    pub use crate::{Context, ServiceContext, SharedContext, SharedServiceContext};

    pub use axum::{extract::Path, http, response::IntoResponse, Json, TypedHeader};
    pub use serde::{Deserialize, Serialize};
    pub use std::borrow::Cow;
    pub use time::format_description::well_known::Iso8601;
    pub use time::OffsetDateTime;
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

#[test]
#[ignore]
fn playground() {
    use ed25519_dalek::Signer;
    use gram::*;

    struct Seed {
        content: String,
        keypair: ed25519_dalek::SigningKey,
        alias: String,
        children: Option<Vec<Seed>>,
    }
    struct Author {
        keypair: ed25519_dalek::SigningKey,
        alias: String,
    }
    fn seed_to_gram(seed: Seed, parent_id: Option<String>) -> Gram {
        let created_at = OffsetDateTime::now_utc();
        let mime = "text/html".to_string();
        let author_pubkey =
            data_encoding::HEXLOWER_PERMISSIVE.encode(&seed.keypair.verifying_key().to_bytes()[..]);
        let json = serde_json::to_string(&serde_json::json!([
            0,
            author_pubkey,
            created_at.unix_timestamp(),
            seed.content,
            mime,
            parent_id
        ]))
        .unwrap();
        let id = blake3::hash(json.as_bytes());

        let sig = seed.keypair.sign(id.as_bytes()).to_bytes();
        let sig = data_encoding::HEXLOWER_PERMISSIVE.encode(&sig[..]);

        let id = data_encoding::HEXLOWER_PERMISSIVE.encode(id.as_bytes());
        Gram {
            id,
            created_at,
            content: seed.content.to_string(),
            mime,
            parent_id,
            author_pubkey,
            author_alias: Some(seed.alias),
            sig,
        }
    }
    fn seeds_to_gram(out: &mut Vec<Gram>, parent_id: Option<String>, seeds: Vec<Seed>) {
        for mut seed in seeds.into_iter() {
            let children = seed.children.take();
            let gram = seed_to_gram(seed, parent_id.clone());
            let parent_id = gram.id.clone();
            out.push(gram);
            if let Some(children) = children {
                seeds_to_gram(out, Some(parent_id), children);
            }
        }
    }
    let mut out = vec![];

    let authors = [
        Author {
            keypair: ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()),
            alias: "use1".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()),
            alias: "fideroth".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()),
            alias: "the_i18n_man".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()),
            alias: "wgt".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()),
            alias: "ftw".to_string(),
        },
    ];
    seeds_to_gram(
        &mut out,
        None,
        vec![Seed {
            content: "I wan't you to know, I wan't you to know that I'm awake.".to_string(),
            keypair: authors[0].keypair.clone(),
            alias: authors[0].alias.clone(),
            children: Some(vec![
                Seed {
                    content: "And I hope you're asleep.".to_string(),
                    keypair: authors[1].keypair.clone(),
                    alias: authors[1].alias.clone(),
                    children: Some(vec![Seed {
                        content: "*air guitars madly*".to_string(),
                        keypair: authors[0].keypair.clone(),
                        alias: authors[0].alias.clone(),
                        children: Some(vec![Seed {
                            content: "*sads doggly*".to_string(),
                            keypair: authors[1].keypair.clone(),
                            alias: authors[1].alias.clone(),
                            children: None,
                        }]),
                    }]),
                },
                Seed {
                    content: "What gives?".to_string(),
                    keypair: authors[2].keypair.clone(),
                    alias: authors[2].alias.clone(),
                    children: Some(vec![Seed {
                        content: "What doesn't?".to_string(),
                        keypair: authors[3].keypair.clone(),
                        alias: authors[3].alias.clone(),
                        children: None,
                    }]),
                },
                Seed {
                    content: "Stop redditing!!!".to_string(),
                    keypair: authors[4].keypair.clone(),
                    alias: authors[4].alias.clone(),
                    children: None,
                },
            ]),
        }],
    );

    println!("{out:#?}");
    for Gram {
        id,
        content,
        mime,
        parent_id,
        sig,
        author_pubkey,
        author_alias,
        ..
    } in out
    {
        println!(
            r#"            ,(
            '\x{id}'::bytea
            ,$${content}$$
            ,'{mime}'
            ,{}
            ,'\x{sig}'::bytea
            ,'\x{author_pubkey}'::bytea
            ,'{}'
            ,'{}'
        )"#,
            if let Some(id) = parent_id {
                format!("'\\x{id}'::bytea")
            } else {
                "NULL".to_string()
            },
            author_alias.as_ref().unwrap(),
            format!("{}@aggy.news", author_alias.as_ref().unwrap()),
        )
    }
}
