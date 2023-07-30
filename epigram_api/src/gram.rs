use crate::interlude::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Gram {
    pub id: String,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub created_at: time::OffsetDateTime,
    pub content: String,
    pub mime: String,
    pub parent_id: Option<String>,

    pub author_pubkey: String,
    pub author_alias: Option<String>,

    pub sig: String,
}

mod get;

pub const TAG: common::Tag = common::Tag {
    name: "gram",
    desc: "bleh",
};

pub fn router() -> axum::Router<SharedContext> {
    axum::Router::new().merge(EndpointWrapper::new(get::GetGram))
}

pub fn components(
    builder: utoipa::openapi::ComponentsBuilder,
) -> utoipa::openapi::ComponentsBuilder {
    let builder = get::GetGram::components(builder);

    builder.schemas_from_iter([<Gram as ToSchema>::schema()])
}

pub fn paths(
    builder: utoipa::openapi::PathsBuilder,
    prefix_path: &str,
) -> utoipa::openapi::PathsBuilder {
    [(get::GetGram::PATH, get::GetGram::path_item())]
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
        "f4776309475fa7460e046126bd8d9453a88cb4548bd35e5f7cfafbf9a0ecb64d5";
    pub const GRAM_02_ID: &'static str =
        "fe20b19235696d1469fd00f44a73de9111f0983227682284fa7526b66c19cded7";
    pub const GRAM_03_ID: &'static str =
        "f2d7ebd96468b0e889887251864c337a8ee042200cd92d93fa77ec6de44049fc0";
    pub const GRAM_04_ID: &'static str =
        "fcc212c4940c8e5872f67493f14c7817bf0e144d34b13d798a116cd606b58c616";

    pub static GRAM_01: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_01_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "I wan't you to know, I wan't you to know that I'm awake.".into(),
        mime: "text/html".into(),
        parent_id: None,
        author_pubkey: "fc993d470f9e138ad4c94bb897e2733f6314e38e9dcb58e661d224234b55c7b98".into(),
        author_alias: Some("use1".into()),
        sig: "f4574a3daaffb3a29704ee1c9937217a4a72f96afc8c4f6e9de5ea7bab85b0b6fa05c692efb380ea92e9fc6058cd683b105bc9245222cb8bef2572dbad6075d09".into(),
    }
    });
    pub static GRAM_02: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_02_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "And I hope you're asleep.".into(),
        mime: "text/html".into(),
        parent_id: Some("f4776309475fa7460e046126bd8d9453a88cb4548bd35e5f7cfafbf9a0ecb64d5".into()),
        author_pubkey: "f108a880634a69715e6d5ccb79888530fe2a204037e5d917d9f750576a084d1a3".into(),
        author_alias: Some("fideroth".into()),
        sig: "fddf5cf0fc11586706931a2ed25cd5dace45db6a9257fbc8f242cd0e98433c6854961107c01fc788b7db5ce0f97944b333e874a643cc130b6723dc29779571f0f".into(),
    }
    });
    pub static GRAM_03: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_03_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "*air guitars madly*".into(),
        mime: "text/html".into(),
        parent_id: Some("fe20b19235696d1469fd00f44a73de9111f0983227682284fa7526b66c19cded7".into()),
        author_pubkey: "fc993d470f9e138ad4c94bb897e2733f6314e38e9dcb58e661d224234b55c7b98".into(),
        author_alias: Some("use1".into()),
        sig: "f42929747078a1e4e5301a7dc8b1092ee8fa091770282d9ddfe3d55051e5a9cee32d2d8a94f4980ccbacee36044fb4be4875def17ae5386653423874e8a9ea208".into(),
    }
    });
    pub static GRAM_04: Lazy<Gram> = Lazy::new(|| {
        Gram {
        id: GRAM_04_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "*sads doggly*".into(),
        mime: "text/html".into(),
        parent_id: Some("f2d7ebd96468b0e889887251864c337a8ee042200cd92d93fa77ec6de44049fc0".into()),
        author_pubkey: "f108a880634a69715e6d5ccb79888530fe2a204037e5d917d9f750576a084d1a3".into(),
        author_alias: Some("fideroth".into()),
        sig: "fc7378d3ee55f7aab0cd284d3a718a62ab20829d08ea0589a12cbfcf6321b5b010bc3f761fbe33ac420cf7a4b69b6dfb39aa64fddf27c5abf273873f007a4b80a".into(),
    }
    });
}
