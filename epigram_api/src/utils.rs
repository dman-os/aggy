use crate::interlude::*;

pub use list_request::*;
mod list_request;

pub fn id_for_gram(
    pub_key_multibase: &str,
    created_at: OffsetDateTime,
    content: &str,
    coty: &str,
    parent_id: Option<&str>,
) -> blake3::Hash {
    let json = serde_json::to_string(&serde_json::json!([
        0,
        pub_key_multibase,
        created_at.unix_timestamp(),
        content,
        coty,
        parent_id
    ]))
    .unwrap();
    blake3::hash(json.as_bytes())
}

pub fn id_and_sig_for_gram(
    keypair: &ed25519_dalek::SigningKey,
    created_at: OffsetDateTime,
    content: &str,
    coty: &str,
    parent_id: Option<&str>,
) -> (blake3::Hash, ed25519_dalek::Signature) {
    use ed25519_dalek::Signer;
    let author_pubkey =
        common::utils::encode_hex_multibase(&keypair.verifying_key().to_bytes()[..]);
    let id = id_for_gram(author_pubkey.as_str(), created_at, content, coty, parent_id);
    let sig = keypair.sign(id.as_bytes());
    (id, sig)
}

pub fn hex_id_and_sig_for_gram(
    keypair: &ed25519_dalek::SigningKey,
    created_at: OffsetDateTime,
    content: &str,
    coty: &str,
    parent_id: Option<&str>,
) -> (String, String) {
    let (id, sig) = id_and_sig_for_gram(keypair, created_at, content, coty, parent_id);
    (
        common::utils::encode_hex_multibase(id.as_bytes()),
        common::utils::encode_hex_multibase(sig.to_bytes()),
    )
}

pub mod testing {

    use common::utils::testing::{TestContext, TestDb};
    use deps::*;

    pub const SERVICE_SECRET: &str = "public square";

    pub fn state_fn_service(testing: &TestContext) -> crate::SharedServiceContext {
        crate::SharedServiceContext(crate::ServiceContext(state_fn(testing)))
    }

    pub async fn cx_fn_service(
        test_name: &'static str,
    ) -> (TestContext, crate::SharedServiceContext) {
        let testing = TestContext::new(
            test_name.into(),
            [("epigram".to_string(), test_db(test_name).await)],
        );
        let cx = state_fn_service(&testing);
        (testing, cx)
    }

    pub async fn test_db(test_name: &'static str) -> TestDb {
        dotenvy::dotenv().ok();
        let db_name = test_name.replace("::tests::", "").replace("::", "_");
        let db_name = format!("epigram_{db_name}");
        TestDb::new(
            db_name,
            std::path::Path::new(&common::utils::get_env_var("EPIGRAM_API_ROOT_PATH").unwrap()),
        )
        .await
    }

    pub fn state_fn(testing: &TestContext) -> crate::SharedContext {
        std::sync::Arc::new(crate::Context {
            db: crate::Db::Pg {
                db_pool: testing.pools["epigram"].pool.clone(),
            },
            config: crate::Config {
                pass_salt_hash: b"sea brine".to_vec(),
                argon2_conf: argon2::Config::default(),
                auth_token_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                web_session_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                service_secret: SERVICE_SECRET.to_string(),
            },
        })
    }

    pub async fn cx_fn(test_name: &'static str) -> (TestContext, crate::SharedContext) {
        let testing = TestContext::new(
            test_name.into(),
            [("epigram".to_string(), test_db(test_name).await)],
        );
        let cx = state_fn(&testing);
        (testing, cx)
    }
}
