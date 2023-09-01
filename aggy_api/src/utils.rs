pub use list_request::*;
mod list_request;

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
            // FIXME: build the test_dbs in parallel
            [
                ("aggy".to_string(), test_db(test_name).await),
                (
                    "epigram".to_string(),
                    epigram_api::utils::testing::test_db(test_name).await,
                ),
            ],
            [],
        );
        let cx = state_fn_service(&testing);
        (testing, cx)
    }

    pub async fn test_db(test_name: &'static str) -> TestDb {
        dotenvy::dotenv().ok();
        let db_name = test_name.replace("::tests::", "_").replace("::", "_");
        TestDb::new(
            db_name,
            std::path::Path::new(&common::utils::get_env_var("AGGY_API_ROOT_PATH").unwrap()),
        )
        .await
    }

    pub fn state_fn_with_epigram(
        // db_pool: sqlx::postgres::PgPool,
        // epigram_cx: epigram_api::SharedContext,
        testing: &TestContext,
    ) -> crate::SharedContext {
        std::sync::Arc::new(crate::Context {
            db: crate::Db::Pg {
                db_pool: testing.pg_pools["aggy"].pool.clone(),
            },
            config: crate::Config {
                pass_salt_hash: b"sea brine".to_vec(),
                argon2_conf: argon2::Config::default(),
                auth_token_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                web_session_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                service_secret: SERVICE_SECRET.to_string(),
            },
            epigram: Box::new(epigram_api::InProcClient {
                cx: epigram_api::utils::testing::state_fn(testing),
            }),
        })
    }

    pub fn state_fn(
        // db_pool: sqlx::postgres::PgPool,
        // epigram_cx: epigram_api::SharedContext,
        testing: &TestContext,
    ) -> crate::SharedContext {
        std::sync::Arc::new(crate::Context {
            db: crate::Db::Pg {
                db_pool: testing.pg_pools["aggy"].pool.clone(),
            },
            config: crate::Config {
                pass_salt_hash: b"sea brine".to_vec(),
                argon2_conf: argon2::Config::default(),
                auth_token_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                web_session_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
                service_secret: SERVICE_SECRET.to_string(),
            },
            epigram: Box::new(epigram_api::HttpClient {}),
        })
    }

    pub async fn cx_fn_with_epigram(
        test_name: &'static str,
    ) -> (TestContext, crate::SharedContext) {
        let testing = TestContext::new(
            test_name.into(),
            [
                ("aggy".to_string(), test_db(test_name).await),
                (
                    "epigram".to_string(),
                    epigram_api::utils::testing::test_db(test_name).await,
                ),
            ],
            [],
        );
        let cx = state_fn_with_epigram(&testing);
        (testing, cx)
    }

    pub async fn cx_fn(test_name: &'static str) -> (TestContext, crate::SharedContext) {
        let testing = TestContext::new(
            test_name.into(),
            [("aggy".to_string(), test_db(test_name).await)],
            [],
        );
        let cx = state_fn(&testing);
        (testing, cx)
    }
}
