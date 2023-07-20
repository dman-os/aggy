use crate::interlude::*;

#[derive(Debug)]
pub struct Session {
    pub token: String,
    pub user_id: uuid::Uuid,
    pub expires_at: time::OffsetDateTime,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Clone)]
pub enum Resource {
    User { id: uuid::Uuid },
    Users,
    WebSession { id: uuid::Uuid },
    WebSessions,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Read,
    Write,
    Delete,
}

#[derive(Debug, Clone, Copy)]
pub enum Role {
    SuperAdmin,
    Service,
}

pub const TAG: common::Tag = common::Tag {
    name: "auth",
    desc: "The authentication and authorization services.",
};

pub mod authenticate;
pub mod authorize;
pub mod authorize_service;

pub fn router() -> axum::Router<SharedContext> {
    axum::Router::new().merge(EndpointWrapper::new(authenticate::Authenticate))
}

pub fn components(
    builder: utoipa::openapi::ComponentsBuilder,
) -> utoipa::openapi::ComponentsBuilder {
    let builder = authenticate::Authenticate::components(builder);
    builder
}

pub fn paths(
    builder: utoipa::openapi::PathsBuilder,
    prefix_path: &str,
) -> utoipa::openapi::PathsBuilder {
    builder.path(
        format!(
            "{prefix_path}{}",
            common::axum_path_str_to_openapi(authenticate::Authenticate::PATH)
        ),
        authenticate::Authenticate::path_item(),
    )
}

pub mod testing {
    pub const USER_01_SESSION: &str = "9d827d5c-15bd-413c-9431-39ff96155d7b";
    pub const USER_04_SESSION: &str = "ebd3b465-be17-4077-bc4a-add9f76b5028";
}
