#![allow(clippy::single_component_path_imports, clippy::let_and_return)]

#[cfg(feature = "dylink")]
#[allow(unused_imports)]
use dylink;

mod interlude {
    pub use deps::*;

    pub use crate::{Context, ServiceContext, SharedContext, SharedServiceContext};

    pub use axum::{extract::Path, http, response::IntoResponse, Json, TypedHeader};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::json;
    pub use sqlx::FromRow;
    pub use std::borrow::Cow;
    pub use time::format_description::well_known::Iso8601;
    pub use time::OffsetDateTime;
    pub use tracing::{debug, error, info, trace, warn};
    pub use utoipa::ToSchema;
    pub use uuid::Uuid;
    pub use validator::Validate;

    pub use common::utils::default;
    pub use common::{
        utils::{CHeapStr, ValidationErrors},
        AuthedUid, AuthenticatedEndpoint, Authorize, DocumentedEndpoint, Endpoint, EndpointWrapper,
        ErrorResponse, HttpEndpoint, HttpResponse, Method, RedisPool, Ref, StatusCode, Tag,
    };

    pub type BearerToken = axum::headers::Authorization<axum::headers::authorization::Bearer>;
    pub type DiscardBody = axum::extract::BodyStream;

    #[cfg(test)]
    pub use crate::utils::testing::*;
    #[cfg(test)]
    pub use common::utils::testing::*;
}
use interlude::*;

pub mod connect;
pub mod event;
pub mod utils;

use crate::utils::*;

use utoipa::openapi;

#[derive(Debug)]
pub struct Config {
    pub pass_salt_hash: Vec<u8>,
    pub argon2_conf: argon2::Config<'static>,
    pub auth_token_lifespan: time::Duration,
    pub web_session_lifespan: time::Duration,
    pub service_secret: String,
    pub event_hose_redis_channel: String,
}

#[derive(Debug)]
pub struct Context {
    pub config: Config,
    pub db: Db,
    pub redis: RedisPool,
    pub sw: connect::Switchboard,
}

#[derive(Debug)]
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
    axum::Router::new()
        .route("/", axum::routing::get(connect::handler))
        .with_state(state)
}

pub struct ApiDoc;
impl utoipa::OpenApi for ApiDoc {
    fn openapi() -> openapi::OpenApi {
        //
        let mut openapi = openapi::OpenApiBuilder::new()
            .info(
                openapi::InfoBuilder::new()
                    .title("qtrunk_api")
                    // .version(build::PKG_VERSION)
                    .description(Some(format!(
                        r#"{}
                        "#,
                        "qtrunk is an experiment"
                    )))
                    .build(),
            )
            .paths({
                let builder = openapi::path::PathsBuilder::new();
                // let builder = gram::paths(builder, "/qtrunk"); //FIXME: make this dyamic
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
                        <common::utils::SortingOrder as utoipa::ToSchema>::schema(),
                        <common::utils::ValidationErrors as utoipa::ToSchema>::schema(),
                        <common::utils::ValidationErrorsKind as utoipa::ToSchema>::schema(),
                        <common::utils::ValidationError as utoipa::ToSchema>::schema(),
                    ]);
                // let builder = gram::components(builder);
                builder.build()
            }))
            // .tags(Some([gram::TAG.into(), common::DEFAULT_TAG.into()]))
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

#[async_trait::async_trait]
pub trait Client {}

pub struct InProcClient {
    pub cx: SharedContext,
}

#[async_trait::async_trait]
impl Client for InProcClient {}

pub struct HttpClient {}

#[async_trait::async_trait]
impl Client for HttpClient {}

#[test]
#[ignore]
fn gen_keypair() {
    let keypair = k256::schnorr::SigningKey::random(&mut rand::thread_rng());
    let prikey = data_encoding::HEXLOWER.encode(&keypair.to_bytes());
    let pubkey = keypair.verifying_key();
    let pubkey = data_encoding::HEXLOWER.encode(&pubkey.to_bytes());
    println!("prikey: {prikey}");
    println!("pubkey: {pubkey}");
}

#[test]
#[ignore]
fn gen_events() {
    use crate::event::*;
    struct Seed {
        prikey: String,
        kind: u16,
        created_at: OffsetDateTime,
        tags: Vec<Vec<String>>,
        content: String,
        replies: Vec<Seed>,
    }
    fn seed_to_event(seed: Seed, parent_id: Option<&str>) -> Vec<Event> {
        let Seed {
            prikey,
            kind,
            content,
            mut tags,
            created_at,
            replies,
        } = seed;
        let prikey = data_encoding::HEXLOWER.decode(prikey.as_bytes()).unwrap();
        let prikey = k256::schnorr::SigningKey::from_bytes(&prikey[..]).unwrap();
        let pubkey = prikey.verifying_key().to_bytes();
        let pubkey = data_encoding::HEXLOWER.encode(&pubkey[..]);
        if let Some(parent_id) = parent_id {
            tags.push(vec!["e".into(), parent_id.into()])
        }
        let (id, sig) = hex_id_and_sig_for_event(
            &prikey,
            pubkey.as_str(),
            created_at,
            kind,
            &tags,
            content.as_str(),
        );
        [Event {
            id: id.clone(),
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig,
        }]
        .into_iter()
        .chain(
            replies
                .into_iter()
                .map(|seed| seed_to_event(seed, Some(&id[..])))
                .flatten(),
        )
        .collect()
    }
    let pubkeys = vec![
        "767ab216ccc49825dc3fc1be67afde623e954d0161799c7d45de8a38988b7de3",
        "f72657e01156d2c9b251111e73d58236dfb7de5ca69e1b53f0a938528f16c265",
        "10ce5bdc0dc22f26fd7142142ba02f8686e9f428ca1b8b04966652d39053334e",
        "07d3cbe0f94c13b75c5c99f9086f101879d769303d0db7b562248dc796297fce",
    ];
    let out = vec![
        Seed {
            prikey: pubkeys[0].into(),
            kind: 1,
            created_at: OffsetDateTime::from_unix_timestamp(1_692_815_146).unwrap(),
            content:
                "the internet would be a better place if it was shut down on tuesdays or the like"
                    .into(),
            tags: vec![],
            replies: default(),
        },
        Seed {
            prikey: pubkeys[1].into(),
            kind: 0,
            created_at: OffsetDateTime::from_unix_timestamp(1_692_815_146).unwrap(),
            content: serde_json::to_string(&json!({
                "name": "bridget",
                "about": "weaponized stink eye",
                "picture": "https://coro.na/virus.png"
            }))
            .unwrap(),
            tags: vec![vec!["p".into(), pubkeys[1].into()]],
            replies: default(),
        },
        Seed {
            prikey: pubkeys[2].into(),
            kind: 1,
            created_at: OffsetDateTime::from_unix_timestamp(1_692_815_146).unwrap(),
            content: "I have information that'll lead to the arrest of Kermit The Frog".into(),
            tags: vec![],
            replies: vec![
                Seed {
                prikey: pubkeys[2].into(),
                kind: 1,
                created_at: OffsetDateTime::from_unix_timestamp(1_692_815_146).unwrap(),
                content: "I'm glad people are paying attention. Information will be released soonTM. Meanwhile, I'll be selling Henson-gate tank-tops and jerseys. Links in my bio".into(),
                tags: vec![],
                replies: vec![],
            },
                Seed {
                prikey: pubkeys[3].into(),
                kind: 1,
                created_at: OffsetDateTime::from_unix_timestamp(1_692_815_146).unwrap(),
                content: "Henson-gate".into(),
                tags: vec![],
                replies: vec![],
            },
            ],
        },
    ]
    .into_iter()
    .map(|seed| seed_to_event(seed, None))
    .flatten()
    .collect::<Vec<_>>();
    for Event {
        id,
        pubkey,
        created_at,
        kind,
        tags,
        content,
        sig,
    } in &out
    {
        println!(
            r#"        ,(
            '\x{id}'::bytea
            ,'\x{pubkey}'::bytea
            ,to_timestamp({})
            ,{kind}
            ,$${}$$::JSONB
            ,$${content}$$
            ,'\x{sig}'::bytea
        )"#,
            created_at.unix_timestamp(),
            serde_json::to_string(&tags).unwrap()
        );
    }
    for (
        idx,
        Event {
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig,
        },
    ) in out.iter().enumerate()
    {
        let idx = idx + 1;
        println!(
            r##"
    pub const EVENT_{idx:02}_ID: &str = "{id}";

    pub static EVENT_{idx:02}: Lazy<Event> = Lazy::new(|| Event {{
        id: EVENT_{idx:02}_ID.into(),
        pubkey: "{pubkey}".into(),
        created_at: OffsetDateTime::from_unix_timestamp({}).unwrap(),
        kind: {kind},
        tags: serde_json::from_str(r#"{}"#).unwrap(),
        content: r#"{content}"#.into(),
        sig: "{sig}".into(),
    }});
            "##,
            created_at.unix_timestamp(),
            serde_json::to_string(&tags).unwrap()
        )
    }
}
