use crate::interlude::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Gram {
    pub id: String,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub created_at: OffsetDateTime,
    pub content: String,
    pub coty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    pub author_pubkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_alias: Option<String>,

    pub sig: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(skip)]
    pub replies: Option<Vec<Gram>>,
}

mod create;
mod get;

pub const TAG: common::Tag = common::Tag {
    name: "gram",
    desc: "bleh",
};

pub fn router() -> axum::Router<SharedContext> {
    axum::Router::new()
        .merge(EndpointWrapper::new(get::GetGram))
        .merge(EndpointWrapper::new(create::CreateGram))
}

pub fn components(
    builder: utoipa::openapi::ComponentsBuilder,
) -> utoipa::openapi::ComponentsBuilder {
    let builder = get::GetGram::components(builder);
    let builder = create::CreateGram::components(builder);

    builder.schemas_from_iter([<Gram as ToSchema>::schema()])
}

pub fn paths(
    builder: utoipa::openapi::PathsBuilder,
    prefix_path: &str,
) -> utoipa::openapi::PathsBuilder {
    [
        (get::GetGram::PATH, get::GetGram::path_item()),
        (create::CreateGram::PATH, create::CreateGram::path_item()),
    ]
    .into_iter()
    .fold(builder, |builder, (path, item)| {
        builder.path(
            format!("{prefix_path}{}", common::axum_path_str_to_openapi(path)),
            item,
        )
    })
}

pub mod testing {
    use super::*;
    use once_cell::sync::Lazy;

    pub const GRAM_01_ID: &'static str =
        "f26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61";
    pub const GRAM_02_ID: &'static str =
        "f863a254a782fae5bcde8629a01a5591a89d1e6bfc531ce5ae4443e149dc29d77";
    pub const GRAM_03_ID: &'static str =
        "fa3bf486c93ed2e6d5d61ecff467670eee74c85942441ddd9422d1139b8044c5b";
    pub const GRAM_04_ID: &'static str =
        "f8e007922fb38461df02aae6409276ba8f9eb39c64066c585ffccb0023146cd79";

    pub static GRAM_01: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_01_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "I wan't you to know, I wan't you to know that I'm awake.".into(),
        coty: "text/html".into(),
        parent_id: None,
        author_pubkey: "f691d917d665d04bb35b65ff896478b9dd59af81ade6c6d7a98d9c19666147c87".into(),
        author_alias: Some("use1".into()),
        sig: "fcc048f2de1d7b3bf0608a3b89a1a71e4f8c8db4049980dca31efe48271ebaabb0572a62bd0346348f5ae09d0b1fd7a530ecab974fc6e474fac46b03127f19802".into(),
        replies: default()
    }
    });
    pub static GRAM_02: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_02_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "And I hope you're asleep.".into(),
        coty: "text/html".into(),
        parent_id: Some(GRAM_01_ID.into()),
        author_pubkey: "fd093f5a4cbc24177a52b4c7b3050c2380f0da88162b84c30f8ff44bbe4e86c77".into(),
        author_alias: Some("fideroth".into()),
        sig: "f6223912f4339bf83829467a32a67cb5e87988f710b65202f86fbb43fbf194f941895e2f5578f205254132ed1d7b1ae8ce712057f19eccccdeb4c20a871fb3e0e".into(),
        replies: default()
    }
    });
    pub static GRAM_03: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_03_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "*air guitars madly*".into(),
        coty: "text/html".into(),
        parent_id: Some(GRAM_02_ID.into()),
        author_pubkey: "f691d917d665d04bb35b65ff896478b9dd59af81ade6c6d7a98d9c19666147c87".into(),
        author_alias: Some("use1".into()),
        sig: "f8f2f73e71d7fc723e4bf0ccafec7ab6726ac0d9c61c6d3d3f4d64419e1a1109fa1502afd8f578100b33e31221fc8cce19ee526f4b6da6424feb6ebd0ccc7be00".into(),
        replies: default()
    }
    });
    pub static GRAM_04: Lazy<Gram> = Lazy::new(|| {
        Gram {
        id: GRAM_04_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "*sads doggly*".into(),
        coty: "text/html".into(),
        parent_id: Some(GRAM_03_ID.into()),
        author_pubkey: "fd093f5a4cbc24177a52b4c7b3050c2380f0da88162b84c30f8ff44bbe4e86c77".into(),
        author_alias: Some("fideroth".into()),
        sig: "f24f497b3bd42f676538fe974cc7e233c74605880b033ec7964db1734eb1aea9d7c530ee9c41376e5cad4c530bf3bb34ef75f9a2a0044ec0d2dd838e1611b2f00".into(),
        replies: default()
    }
    });
}
