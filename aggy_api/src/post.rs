use crate::interlude::*;

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Post {
    pub id: uuid::Uuid,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub updated_at: time::OffsetDateTime,

    pub epigram_id: String,
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>,

    #[schema(example = "hunter2")]
    pub author_username: String,
    pub author_pic_url: Option<String>,
    pub author_pub_key: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(skip)]
    #[schema(value_type = Option<Gram>)]
    pub epigram: Option<epigram_api::gram::Gram>,
}

pub use list::PostSortingField;

pub const TAG: common::Tag = common::Tag {
    name: "post",
    desc: "Manipulate Post objects.",
};

mod create;
mod reply;
// mod delete;
mod get;
mod list;
// mod update;

pub fn router() -> axum::Router<SharedContext> {
    axum::Router::new()
        .merge(EndpointWrapper::new(get::GetPost))
        .merge(EndpointWrapper::new(create::CreatePost))
        .merge(EndpointWrapper::new(reply::Reply))
        // .merge(EndpointWrapper::new(update::UpdatePost))
        .merge(EndpointWrapper::new(list::ListPosts))
    // .merge(EndpointWrapper::new(delete::DeletePost))
}

pub fn components(
    builder: utoipa::openapi::ComponentsBuilder,
) -> utoipa::openapi::ComponentsBuilder {
    let builder = get::GetPost::components(builder);
    let builder = create::CreatePost::components(builder);
    let builder = reply::Reply::components(builder);
    // let builder = update::UpdatePost::components(builder);
    let builder = list::ListPosts::components(builder);
    // let builder = delete::DeletePost::components(builder);
    builder.schemas_from_iter([
        <Post as utoipa::ToSchema>::schema(),
        <epigram_api::gram::Gram as utoipa::ToSchema>::schema(),
        <PostSortingField as utoipa::ToSchema>::schema(),
    ])
}

pub fn paths(
    builder: utoipa::openapi::PathsBuilder,
    prefix_path: &str,
) -> utoipa::openapi::PathsBuilder {
    [
        (get::GetPost::PATH, get::GetPost::path_item()),
        (create::CreatePost::PATH, create::CreatePost::path_item()),
        (reply::Reply::PATH, reply::Reply::path_item()),
        // (update::UpdatePost::PATH, update::UpdatePost::path_item()),
        // (delete::DeletePost::PATH, delete::DeletePost::path_item()),
        (list::ListPosts::PATH, list::ListPosts::path_item()),
    ]
    .into_iter()
    .fold(builder, |builder, (path, item)| {
        builder.path(
            format!("{prefix_path}{}", common::axum_path_str_to_openapi(path)),
            item,
        )
    })
}

// #[cfg(test)]
pub mod testing {
    use deps::*;

    pub const POST_01_ID: uuid::Uuid = uuid::uuid!("a4dac041-b0a4-4afd-a1a6-83ed69c4dfe5");
    pub const POST_02_ID: uuid::Uuid = uuid::uuid!("244018b4-8081-4a93-9828-6e908591bd16");
    pub const POST_03_ID: uuid::Uuid = uuid::uuid!("4829545d-a9ff-4a06-b00f-a22a6ba4c5eb");
    pub const POST_04_ID: uuid::Uuid = uuid::uuid!("d7c222dd-f4bb-4639-ae6e-41c94cc57be1");

    pub const POST_02_EPIGRAM_ID: &'static str =
        "ff1fe48098ee8a9c3de6ad11d132f4bbfa5ddfe1e3ab0608b4a07aacadd4e69b9";
}
