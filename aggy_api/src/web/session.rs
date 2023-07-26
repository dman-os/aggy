use crate::interlude::*;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Session {
    pub id: uuid::Uuid,
    pub ip_addr: std::net::IpAddr,
    #[schema(example = "ViolaWWW")]
    pub user_agent: String,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub expires_at: time::OffsetDateTime,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub updated_at: time::OffsetDateTime,

    pub user_id: Option<uuid::Uuid>,
    pub token: Option<String>,
    #[serde(with = "common::codecs::sane_iso8601::option")]
    pub token_expires_at: Option<time::OffsetDateTime>,
}

pub mod create;
pub mod get;
pub mod update;

pub mod testing {
    pub const USER_01_WEB_SESSION: &str = "13e4cbdf-aa7c-43ca-990c-a8b468d44616";
    pub const USER_04_WEB_SESSION: &str = "0a7f6a02-43a4-4738-b70c-0d66eb24459f";
}
