pub use list_request::*;
mod list_request;

pub mod testing {

    use common::utils::testing::{TestContext, TestDb};
    use deps::*;

    pub const SERVICE_SECRET: &'static str = "public square";

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
