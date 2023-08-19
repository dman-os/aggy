#![allow(clippy::single_component_path_imports, clippy::let_and_return)]

#[cfg(feature = "dylink")]
#[allow(unused_imports)]
use dylink;

mod interlude {
    pub use deps::*;

    pub use crate::{Context, ServiceContext, SharedContext, SharedServiceContext};

    pub use axum::{extract::Path, http, response::IntoResponse, Json, TypedHeader};
    pub use serde::{Deserialize, Serialize};
    pub use sqlx::FromRow;
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
    axum::Router::new()
        .merge(gram::router())
        .with_state(state)
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
                let builder = gram::paths(builder, "/epigram"); //FIXME: make this dyamic
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
                let builder = gram::components(builder);
                builder.build()
            }))
            .tags(Some([gram::TAG.into(), common::DEFAULT_TAG.into()]))
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
pub trait Client {
    async fn get_gram(
        &self,
        request: crate::gram::get::Request,
    ) -> Result<crate::gram::get::Response, Box<dyn std::error::Error>>;
    async fn create_gram(
        &self,
        request: crate::gram::create::Request,
    ) -> Result<crate::gram::create::Response, Box<dyn std::error::Error>>;
}

pub struct InProcClient {
    pub cx: SharedContext,
}

#[async_trait::async_trait]
impl Client for InProcClient {
    async fn get_gram(
        &self,
        request: crate::gram::get::Request,
    ) -> Result<crate::gram::get::Response, Box<dyn std::error::Error + 'static>> {
        crate::gram::get::GetGram
            .handle(&self.cx, request)
            .await
            .map_err(|err| err.into())
    }
    async fn create_gram(
        &self,
        request: crate::gram::create::Request,
    ) -> Result<crate::gram::create::Response, Box<dyn std::error::Error + 'static>> {
        crate::gram::create::CreateGram
            .handle(&self.cx, request)
            .await
            .map_err(|err| err.into())
    }
}

pub struct HttpClient {}

#[async_trait::async_trait]
impl Client for HttpClient {
    async fn get_gram(
        &self,
        _: crate::gram::get::Request,
    ) -> Result<crate::gram::get::Response, Box<dyn std::error::Error + 'static>> {
        todo!("epigram_api::HttpClient is not yet implemented")
    }
    async fn create_gram(
        &self,
        _: crate::gram::create::Request,
    ) -> Result<crate::gram::create::Response, Box<dyn std::error::Error + 'static>> {
        todo!("epigram_api::HttpClient is not yet implemented")
    }
}

#[test]
#[ignore]
fn gen_grams() {
    use gram::*;

    struct Seed {
        content: String,
        keypair: ed25519_dalek::SigningKey,
        alias: String,
        replies: Option<Vec<Seed>>,
    }
    struct Author {
        keypair: ed25519_dalek::SigningKey,
        alias: String,
    }
    fn seed_to_gram(seed: Seed, parent_id: Option<String>) -> Gram {
        let created_at = OffsetDateTime::from_unix_timestamp(1_691_479_928).unwrap();
        let coty = "text/html".to_string();
        let (id, sig) = crate::utils::id_and_sig_for_gram(
            &seed.keypair,
            created_at,
            seed.content.as_str(),
            coty.as_str(),
            parent_id.as_deref(),
        );
        // NOTE: we don't use multibase encoding
        let author_pubkey =
            data_encoding::HEXLOWER_PERMISSIVE.encode(seed.keypair.verifying_key().as_bytes());
        let sig = data_encoding::HEXLOWER_PERMISSIVE.encode(&sig.to_bytes()[..]);
        let id = data_encoding::HEXLOWER_PERMISSIVE.encode(id.as_bytes());
        Gram {
            id,
            created_at,
            content: seed.content.to_string(),
            coty,
            parent_id,
            author_pubkey,
            author_alias: Some(seed.alias),
            sig,
            replies: default(),
            reply_count: Some(0),
        }
    }
    fn seeds_to_gram(out: &mut Vec<Gram>, parent_id: Option<String>, seeds: Vec<Seed>) {
        for mut seed in seeds.into_iter() {
            let children = seed.replies.take();
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
    let aggy_authors = [
        Author {
            keypair: ed25519_dalek::SigningKey::from_bytes(
                common::utils::decode_hex_multibase(
                    "feb28ec6fa7d60b719af82e4de57391dfda3fd354a344acd5f4ae143ca6554d3e",
                )
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            ),
            alias: "sabrina".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::from_bytes(
                common::utils::decode_hex_multibase(
                    "f7ceffe6e9dd0cba3bd2cd362e472b0b94d0f4b1417c665f7249967ebdc7fd6a0",
                )
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            ),
            alias: "archie".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::from_bytes(
                common::utils::decode_hex_multibase(
                    "f223c52751e99d3acfa7dc2a9185fe7b6ec8f3acbc5503ae9f3815033e1f04846",
                )
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            ),
            alias: "betty".to_string(),
        },
        Author {
            keypair: ed25519_dalek::SigningKey::from_bytes(
                common::utils::decode_hex_multibase(
                    "f359b2f5d06e233765fc2afcc51e39b716b0d790d4233f8f07e1ebb08a3de8223",
                )
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            ),
            alias: "veronica".to_string(),
        },
    ];
    seeds_to_gram(
        &mut out,
        None,
        vec![
            Seed {
                content: "I wan't you to know, I wan't you to know that I'm awake.".to_string(),
                keypair: authors[0].keypair.clone(),
                alias: authors[0].alias.clone(),
                replies: Some(vec![
                    Seed {
                        content: "And I hope you're asleep.".to_string(),
                        keypair: authors[1].keypair.clone(),
                        alias: authors[1].alias.clone(),
                        replies: Some(vec![Seed {
                            content: "*air guitars madly*".to_string(),
                            keypair: authors[0].keypair.clone(),
                            alias: authors[0].alias.clone(),
                            replies: Some(vec![Seed {
                                content: "*sads doggly*".to_string(),
                                keypair: authors[1].keypair.clone(),
                                alias: authors[1].alias.clone(),
                                replies: None,
                            }]),
                        }]),
                    },
                    Seed {
                        content: "What gives?".to_string(),
                        keypair: authors[2].keypair.clone(),
                        alias: authors[2].alias.clone(),
                        replies: Some(vec![Seed {
                            content: "What doesn't?".to_string(),
                            keypair: authors[3].keypair.clone(),
                            alias: authors[3].alias.clone(),
                            replies: None,
                        }]),
                    },
                    Seed {
                        content: "Stop redditing!!!".to_string(),
                        keypair: authors[4].keypair.clone(),
                        alias: authors[4].alias.clone(),
                        replies: None,
                    },
                ]),
            },
            Seed {
                content: r#"<a href="https://simple.news/p/atlantis-resurface">Atlantis resurfaces 20 miles off the coast of Hong Kong!</a>
<p>
This is an ongoing story. Please abstain from moralspeech or alterjecting. 

Make sure to make use of pubkeys registered on the Bloodchain as per JURISPRUDENCE-COMMIT-9becb3c12. All unregistered pubkeys will be held liabale for any casualites and damage in case of flamewars.</p>"#
                    .to_string(),
                keypair: aggy_authors[0].keypair.clone(),
                alias: aggy_authors[0].alias.clone(),
                replies: Some(vec![
                    Seed {
                        content: "I'd like to know what the probablities of this being a psyop are considering international relations and the situation in the pacific?".to_string(),
                        keypair: aggy_authors[1].keypair.clone(),
                        alias: aggy_authors[1].alias.clone(),
                        replies: Some(vec![Seed {
                            content: "95% a psyop.".to_string(),
                            keypair: aggy_authors[2].keypair.clone(),
                            alias: aggy_authors[2].alias.clone(),
                            replies: Some(vec![Seed {
                                content: "I was hoping for paragraphs.".to_string(),
                                keypair: aggy_authors[1].keypair.clone(),
                                alias: aggy_authors[1].alias.clone(),
                                replies: Some(vec![Seed {
                                    content: "No one here knows enough for paragraphs.".to_string(),
                                    keypair: aggy_authors[2].keypair.clone(),
                                    alias: aggy_authors[2].alias.clone(),
                                    replies: None,
                                }]),
                            }]),
                        }]),
                    },
                    Seed {
                        content: "How do you do fellow terrestrials? We come in peace.".to_string(),
                        keypair: aggy_authors[3].keypair.clone(),
                        alias: aggy_authors[3].alias.clone(),
                        replies: Some(vec![Seed {
                            content: "How're you able to access this messageboard?".to_string(),
                            keypair: aggy_authors[0].keypair.clone(),
                            alias: aggy_authors[0].alias.clone(),
                            replies: Some(vec![
                                Seed {
                                    content: "Atlantis runs on a UNIX derivate.".to_string(),
                                    keypair: aggy_authors[3].keypair.clone(),
                                    alias: aggy_authors[3].alias.clone(),
                                    replies: None,
                                }
                            ]),
                        }]),
                    },
                ]),
            },
            Seed {
                content: r#"<a href="https://aggy.news/p/a0c78830-d6c5-4133-af47-daac110aeb2c.txt">I suspect my wife of YDL membership</a>

<p>I first started to notice the signs a few weeks ago after I discovred somne inconsistency in my terminal history. Note: I'm currently employed an employee of Alphaborg at their Youtube division. Any advice is appreciated.</p>"#
                    .to_string(),
                keypair: aggy_authors[1].keypair.clone(),
                alias: aggy_authors[1].alias.clone(),
                replies: Some(vec![
                    Seed {
                        content: "NOTE: Thread has been shut down due to unsanctioned flamewar.".to_string(),
                        keypair: aggy_authors[2].keypair.clone(),
                        alias: aggy_authors[2].alias.clone(),
                        replies: None,
                    }
                ]),
            },
            Seed {
                content: r#"<a href="nncp://857893/8471291/7583921748203.txt">Tokyo report shows record numbers of discarded limbs infesting underways</a>"#
                    .to_string(),
                keypair: aggy_authors[1].keypair.clone(),
                alias: aggy_authors[1].alias.clone(),
                replies: None,
            },
            Seed {
                content: r#"<a href="https://nil.null/89897898-rem-adware-danger">REM sleep adware considered dangerous</a>"#
                    .to_string(),
                keypair: aggy_authors[2].keypair.clone(),
                alias: aggy_authors[2].alias.clone(),
                replies: None,
            },
            Seed {
                content: r#"<a href="https://arxiv.org/abs/31415.193">P=NP in 9 dimensions</a>"#
                    .to_string(),
                keypair: aggy_authors[3].keypair.clone(),
                alias: aggy_authors[3].alias.clone(),
                replies: None,
            },
        ],
    );

    println!("{out:#?}");
    for Gram {
        id,
        content,
        coty,
        parent_id,
        sig,
        author_pubkey,
        author_alias,
        created_at,
        ..
    } in out
    {
        println!(
            r#"        ,(
            '\x{id}'::bytea
            ,to_timestamp({})
            ,$${content}$$
            ,'{coty}'
            ,{}
            ,'\x{sig}'::bytea
            ,'\x{author_pubkey}'::bytea
            ,'{}'
            ,'{}'
        )"#,
            created_at.unix_timestamp(),
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

#[test]
#[ignore]
fn lettre_test() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            use lettre::message::header::ContentType;
            // use lettre::transport::smtp::authentication::Credentials;
            use lettre::{AsyncSmtpTransport, AsyncTransport, Message};

            let email = Message::builder()
                .from("NoBody <nobody@domain.tld>".parse().unwrap())
                .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
                .to("Hei <hei@domain.tld>".parse().unwrap())
                .subject("Happy new year")
                .header(ContentType::TEXT_PLAIN)
                .body(String::from("Be happy!"))
                .unwrap();

            let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous("0")
                .port(2500)
                .build();

            // Send the email
            match mailer.send(email).await {
                Ok(_) => println!("Email sent successfully!"),
                Err(e) => panic!("Could not send email: {:?}", e),
            }
        });
}
