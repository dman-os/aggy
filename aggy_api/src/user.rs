use crate::interlude::*;

use once_cell::sync::Lazy;

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct User {
    pub id: uuid::Uuid,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub updated_at: time::OffsetDateTime,
    #[schema(example = "alice@example.com")]
    pub email: Option<String>,
    #[schema(example = "hunter2")]
    pub username: String,
    pub pic_url: Option<String>,
    pub pub_key: String,
}

pub use list::UserSortingField;

pub static USERNAME_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^[a-zA-Z0-9]+([_-]?[a-zA-Z0-9])*$").unwrap());

pub const TAG: common::Tag = common::Tag {
    name: "user",
    desc: "Manipulate User objects.",
};

mod create;
mod delete;
mod get;
mod list;
mod update;

pub fn router() -> axum::Router<SharedContext> {
    axum::Router::new()
        .merge(EndpointWrapper::new(get::GetUser))
        .merge(EndpointWrapper::new(create::CreateUser))
        .merge(EndpointWrapper::new(update::UpdateUser))
        .merge(EndpointWrapper::new(list::ListUsers))
        .merge(EndpointWrapper::new(delete::DeleteUser))
}

pub fn components(
    builder: utoipa::openapi::ComponentsBuilder,
) -> utoipa::openapi::ComponentsBuilder {
    let builder = get::GetUser::components(builder);
    let builder = create::CreateUser::components(builder);
    let builder = update::UpdateUser::components(builder);
    let builder = list::ListUsers::components(builder);
    let builder = delete::DeleteUser::components(builder);
    builder
        .schemas_from_iter([
            <User as utoipa::ToSchema>::schema(),
            <UserSortingField as utoipa::ToSchema>::schema(),
        ])
        .schemas_from_iter(<list::ListUsersRequest as utoipa::ToSchema>::aliases())
        .schemas_from_iter(<list::ListUsersResponse as utoipa::ToSchema>::aliases())
}

pub fn paths(
    builder: utoipa::openapi::PathsBuilder,
    prefix_path: &str,
) -> utoipa::openapi::PathsBuilder {
    [
        (get::GetUser::PATH, get::GetUser::path_item()),
        (create::CreateUser::PATH, create::CreateUser::path_item()),
        (update::UpdateUser::PATH, update::UpdateUser::path_item()),
        (delete::DeleteUser::PATH, delete::DeleteUser::path_item()),
        (list::ListUsers::PATH, list::ListUsers::path_item()),
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

    pub const USER_01_ID: uuid::Uuid = uuid::uuid!("add83cdf-2ab3-443f-84dd-476d7984cf75");
    pub const USER_02_ID: uuid::Uuid = uuid::uuid!("ce4fe993-04d6-462e-af1d-d734fcc9639d");
    pub const USER_03_ID: uuid::Uuid = uuid::uuid!("d437e73f-4610-462c-ab22-f94b76bba83a");
    pub const USER_04_ID: uuid::Uuid = uuid::uuid!("68cf4d43-62d2-4202-8c50-c79a5f4dd1cc");

    pub const USER_01_USERNAME: &str = "sabrina";
    pub const USER_02_USERNAME: &str = "archie";
    pub const USER_03_USERNAME: &str = "betty";
    pub const USER_04_USERNAME: &str = "veronica";

    pub const USER_01_EMAIL: &str = "hex.queen@teen.dj";
    pub const USER_02_EMAIL: &str = "archie1941@poetry.ybn";
    pub const USER_03_EMAIL: &str = "pInXy@melt.shake";
    pub const USER_04_EMAIL: &str = "trekkiegirl@ln.pi";
}
