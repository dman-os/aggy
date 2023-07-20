use crate::interlude::*;

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Session {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub ip_addr: std::net::IpAddr,
    #[schema(example = "ViolaWWW")]
    pub user_agent: String,
    #[schema(example = 1234567)]
    #[serde(with = "time::serde::timestamp")]
    pub expires_at: time::OffsetDateTime,
    #[schema(example = 1234567)]
    #[serde(with = "time::serde::timestamp")]
    pub created_at: time::OffsetDateTime,
    #[schema(example = 1234567)]
    #[serde(with = "time::serde::timestamp")]
    pub updated_at: time::OffsetDateTime,
}

pub mod create;
pub mod get;

pub mod testing {
    pub const USER_01_WEB_SESSION: &str = "13e4cbdf-aa7c-43ca-990c-a8b468d44616";
    pub const USER_04_WEB_SESSION: &str = "0a7f6a02-43a4-4738-b70c-0d66eb24459f";
}
