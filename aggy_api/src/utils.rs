pub use list_request::*;
mod list_request;

#[cfg(test)]
pub mod testing {

    use deps::*;

    pub fn state_fn(testing: &common::utils::testing::TestContext) -> crate::SharedContext {
        std::sync::Arc::new(crate::Context {
            db_pool: testing.db_pool.clone().unwrap(),
            config: crate::Config {
                pass_salt_hash: b"sea brine".to_vec(),
                argon2_conf: argon2::Config::default(),
                auth_token_lifespan: time::Duration::seconds_f64(60. * 60. * 24. * 30.),
            },
        })
    }
}
