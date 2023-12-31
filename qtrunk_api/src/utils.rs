pub mod testing {
    use crate::interlude::*;

    use common::utils::testing::{TestContext, TestDb, TestRedis};

    pub const SERVICE_SECRET: &str = "public square";

    pub async fn test_db(test_name: &'static str) -> TestDb {
        dotenvy::dotenv().ok();
        let db_name = test_name.replace("::tests::", "_").replace("::", "_");
        TestDb::new(
            db_name,
            std::path::Path::new(&common::utils::get_env_var("QTRUNK_API_ROOT_PATH").unwrap()),
        )
        .await
    }

    pub fn state_fn(testing: &TestContext) -> crate::SharedContext {
        std::sync::Arc::new(crate::Context {
            db: crate::Db::Pg {
                db_pool: testing.pg_pools["qtrunk"].pool.clone(),
            },
            redis: testing.redis_pools["default"].pool.clone(),
            config: crate::Config {
                pass_salt_hash: b"sea brine".to_vec(),
                argon2_conf: argon2::Config::default(),
                auth_token_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                web_session_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                service_secret: SERVICE_SECRET.to_string(),
                event_hose_redis_channel: format!("event_hose_{}", testing.test_name),
            },
            sw: default(),
        })
    }

    pub async fn cx_fn(test_name: &'static str) -> (TestContext, crate::SharedContext) {
        let testing = TestContext::new(
            test_name.into(),
            [("qtrunk".to_string(), test_db(test_name).await)],
            [("default".to_string(), TestRedis::new().await)],
        );
        let cx = state_fn(&testing);
        drop(tokio::spawn(crate::connect::start_switchboard(cx.clone())));
        (testing, cx)
    }
}
