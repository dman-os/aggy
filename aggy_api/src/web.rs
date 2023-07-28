use crate::interlude::*;

pub mod session;

pub const TAG: common::Tag = common::Tag {
    name: "web",
    desc: "For exclusive use of the web app.",
};

pub fn router() -> axum::Router<SharedServiceContext> {
    axum::Router::new()
        .merge(EndpointWrapper::new(session::get::GetWebSession))
        .merge(EndpointWrapper::new(session::create::CreateWebSession))
        .merge(EndpointWrapper::new(session::update::UpdateWebSession))
}

pub fn components(
    builder: utoipa::openapi::ComponentsBuilder,
) -> utoipa::openapi::ComponentsBuilder {
    let builder = session::get::GetWebSession::components(builder);
    let builder = session::create::CreateWebSession::components(builder);
    let builder = session::update::UpdateWebSession::components(builder);
    builder.schemas_from_iter([<session::Session as utoipa::ToSchema>::schema()])
}

pub fn paths(
    builder: utoipa::openapi::PathsBuilder,
    prefix_path: &str,
) -> utoipa::openapi::PathsBuilder {
    [
        (
            session::get::GetWebSession::PATH,
            session::get::GetWebSession::path_item(),
        ),
        (
            session::create::CreateWebSession::PATH,
            session::create::CreateWebSession::path_item(),
        ),
        (
            session::update::UpdateWebSession::PATH,
            session::update::UpdateWebSession::path_item(),
        ),
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
    pub const USER_01_SESSION: &str = "9d827d5c-15bd-413c-9431-39ff96155d7b";
    pub const USER_04_SESSION: &str = "ebd3b465-be17-4077-bc4a-add9f76b5028";
}
