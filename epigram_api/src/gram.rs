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

    pub const GRAM_01_ID: &'static str =
        "f43203d798236edc55a20be5de2f9a401514b62e85ad0792fdffe4ca9b1b6a5a0";
    pub const GRAM_02_ID: &'static str =
        "fa52b926f893d3019cdbddbf4c553232c43035b4fe08eb26c31892e3b57b3dfc5";
    pub const GRAM_03_ID: &'static str =
        "f733866a0fd613e6afbd4687d8c5a4116cd298403c31fecf88f84c5961fc53a23";
    pub const GRAM_04_ID: &'static str =
        "f0ae08db70cf1ff1d6d56248e59c6ab3adb1f6e34e3fe3fee61f636cc2f4851e1";

    pub static GRAM_01: Lazy<Gram> = Lazy::new(|| {
        Gram{
        id: GRAM_01_ID.into(),
        created_at: OffsetDateTime::now_utc(),
        content: "I wan't you to know, I wan't you to know that I'm awake.".into(),
        coty: "text/html".into(),
        parent_id: None,
        author_pubkey: "fdc67469e70cbcab49a7840ab1b44d56c7963a7ca24c626ebb792fa7f514f37aa".into(),
        author_alias: Some("use1".into()),
        sig: "f2492541d9c12871570caeecb1aaec6d2197b5e058405393dc624a2299568055ec092fe91ea34a85b99cb45921fbab2999f5aa7f215956e4806f948b49a0d020d".into(),
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
        author_pubkey: "f43366142dc9ce1022ce9ec9deb72b0242ed3e097c8d7531cad8bf982cf9edb7f".into(),
        author_alias: Some("fideroth".into()),
        sig: "f8e5934a66506ec9742c0af2393fcc5158da433012284f071b8c4a47b721cec3db91057f59f3f609bc23eaabe97a55524aff30ff12eb18b1d3cf55eaf981e3601".into(),
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
        author_pubkey: "fdc67469e70cbcab49a7840ab1b44d56c7963a7ca24c626ebb792fa7f514f37aa".into(),
        author_alias: Some("use1".into()),
        sig: "ffae46b531cb4ef8e466baa4d14e0fa8eec249345387717f72b28789c559486ab1b17b88e3bf3f22adafc6654dc8fc606fcbcbfcb82bed4254ccf142ed2c05a05".into(),
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
        author_pubkey: "f43366142dc9ce1022ce9ec9deb72b0242ed3e097c8d7531cad8bf982cf9edb7f".into(),
        author_alias: Some("fideroth".into()),
        sig: "f85b92cd7c5d5e79778cf7698f728b6fec0c34f94300c9c59abbe755f526df461d35819f19d3c3704192e9f2b0cb1dbfeb900710e32c2afcca9a2469d8a832d09".into(),
        replies: default()
    }
    });
}
