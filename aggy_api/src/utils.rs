use crate::interlude::*;

pub use list_request::*;
mod list_request;

pub fn encode_hex_multibase<T: AsRef<[u8]>>(source: T) -> String {
    format!("f{}", hex::encode(source))
}

#[cfg(test)]
pub mod testing {

    use deps::*;

    pub const SERVICE_SECRET: &'static str = "public square";

    pub fn state_fn_service(
        testing: &common::utils::testing::TestContext,
    ) -> crate::SharedServiceContext {
        crate::SharedServiceContext(crate::ServiceContext(state_fn(testing)))
    }

    pub fn state_fn(testing: &common::utils::testing::TestContext) -> crate::SharedContext {
        std::sync::Arc::new(crate::Context {
            db: crate::Db::Pg {
                db_pool: testing.db_pool.clone().unwrap(),
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
}
