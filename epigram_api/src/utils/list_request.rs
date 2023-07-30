use crate::interlude::*;

use serde::{Deserialize, Serialize};

pub use common::utils::{Cursor, SortingField, SortingOrder};

#[derive(Debug, Serialize, Deserialize, validator::Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
#[validate(schema(function = "validate_list_req"))]
// #[aliases(ListUsersRequest = ListRequest<UserSortingField>)]
pub struct ListRequest<S>
where
    S: SortingField + Clone + Copy + Serialize + utoipa::ToSchema<'static>,
{
    #[serde(skip)]
    pub auth_token: Option<BearerToken>,
    #[schema(minimum = 1, maximum = 100)]
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<usize>,
    pub after_cursor: Option<String>,
    pub before_cursor: Option<String>,
    pub filter: Option<String>,
    pub sorting_field: Option<S>,
    pub sorting_order: Option<SortingOrder>,
}

fn validate_list_req<S>(req: &ListRequest<S>) -> Result<(), validator::ValidationError>
where
    S: SortingField + Clone + Copy + Serialize + utoipa::ToSchema<'static>,
{
    common::utils::validate_list_req(
        req.after_cursor.as_ref().map(|s| &s[..]),
        req.before_cursor.as_ref().map(|s| &s[..]),
        req.filter.as_ref().map(|s| &s[..]),
        req.sorting_field,
        req.sorting_order,
    )
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
// #[aliases(ListUsersResponse = ListResponse<User>)]
pub struct ListResponse<T>
where
    T: utoipa::ToSchema<'static>,
{
    pub cursor: Option<String>,
    pub items: Vec<T>,
}
