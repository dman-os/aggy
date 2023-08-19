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
    #[schema(value_type = Option<Vec<Gram>>)]
    pub replies: Option<Vec<Gram>>,
    pub reply_count: Option<i64>,
}

pub mod create;
pub mod get;

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

    pub const GRAM_01_ID: &str =
        "fc6d9d817d53dee6c0ae00205e9f32f6373b23215ddd442a5dce193cce73f5925";
    pub const GRAM_02_ID: &str =
        "f35a356563678440efa1eb44e5cb2036e5e31b9eb6f04ef5df0c70966d5226b12";
    pub const GRAM_03_ID: &str =
        "f6affc96805b62f5a4c47b1ef2cf436910eb4df0253c7226d94406e6ab2771de5";
    pub const GRAM_04_ID: &str =
        "f64eb58f1ee950ea7519039eb39690bd56c94065301246cb5d572b703bdaa6421";

    pub static GRAM_01: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_01_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "I wan't you to know, I wan't you to know that I'm awake.".into(),
        coty: "text/html".into(),
        parent_id: None,
        author_pubkey: "f4ea301616a42cfbbd03f33570038156065fc217a86cdcb993e9fb9b197d08b53".into(),
        author_alias: Some("use1".into()),
        sig: "f06a6016f64de7f22123816cc6a00db5c3d7d62da64fcb42daba234e2f6ecbc4ea6bb1671d035c3ffdbe6ed2a92dafbd5341f1d107557043b8d2fe018f17fbe0e".into(),
        replies: default(),
        reply_count: default(),
    }
    });
    pub static GRAM_02: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_02_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "And I hope you're asleep.".into(),
        coty: "text/html".into(),
        parent_id: Some(GRAM_01_ID.into()),
        author_pubkey: "fe90bb6e011ed9b2607b45c6917405f56b5c793168c578343e353cde94c4b6bed".into(),
        author_alias: Some("fideroth".into()),
        sig: "f519096262a6b214837dae999e8688d265bbed056207bc47fcf30e8a4b526b2bcd0e708f002f7c5d3ead38453a53a40735fb35fc56030902eb9a6eef03df66405".into(),
        replies: default(),
        reply_count: default(),
    }
    });
    pub static GRAM_03: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_03_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "*air guitars madly*".into(),
        coty: "text/html".into(),
        parent_id: Some(GRAM_02_ID.into()),
        author_pubkey: "f4ea301616a42cfbbd03f33570038156065fc217a86cdcb993e9fb9b197d08b53".into(),
        author_alias: Some("use1".into()),
        sig: "f9805011ae871eadbf5ab8e8501c2697731361ce11410d8afa9af696f89ce059f27dbce9bee77dc41e9fa4c44a7adfa02250e4f09911c7bd45302f846ebbeac0e".into(),
        replies: default(),
        reply_count: default(),
    }
    });
    pub static GRAM_04: Lazy<Gram> = Lazy::new(|| {
        Gram {
        id: GRAM_04_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "*sads doggly*".into(),
        coty: "text/html".into(),
        parent_id: Some(GRAM_03_ID.into()),
        author_pubkey: "fe90bb6e011ed9b2607b45c6917405f56b5c793168c578343e353cde94c4b6bed".into(),
        author_alias: Some("fideroth".into()),
        sig: "f8bc68f72d274ad8919a01a62e2b512175fec2be38211de1c760dcd775539f45da0509d725ee4171a8ff4d78a370ae179f857a3ff3c78da0f6cfb6bd9d076990b".into(),
        replies: default(),
        reply_count: default(),
    }
    });
}
