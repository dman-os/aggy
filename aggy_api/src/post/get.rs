use crate::interlude::*;

use super::Post;

use axum::extract::Query;

#[derive(Clone, Copy, Debug)]
pub struct GetPost;

#[derive(Debug)]
pub struct Request {
    // pub auth_token: BearerToken,
    pub id: Uuid,
    pub include_replies: bool,
}

pub type Response = Ref<Post>;

#[derive(Debug, thiserror::Error, Serialize, ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    #[error("post not found at id: {id:?}")]
    NotFound { id: Uuid },
    // #[error("{self:?}")]
    // AccessDenied,
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl Endpoint for GetPost {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    #[tracing::instrument(skip(cx))]
    async fn handle(
        &self,
        cx: &Self::Cx,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let out = match &cx.db {
            crate::Db::Pg { db_pool } => {
                let row = sqlx::query!(
                    r#"
SELECT 
    p.created_at as "created_at!"
    ,p.updated_at as "updated_at!"
    ,p.id as "id"
    ,p.title as "title"
    ,p.url as "url"
    ,p.body as "body"
    ,util.multibase_encode_hex(p.epigram_id) as "epigram_id!"
    ,util.multibase_encode_hex(u.pub_key) as "author_pub_key!"
    ,u.username::TEXT as "author_username!"
    ,u.pic_url as "author_pic_url"
FROM 
    posts.posts as p
        LEFT JOIN
    auth.users as u
        ON (p.author_id = u.id)
WHERE p.id = $1::UUID
                "#,
                    &request.id
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| match err {
                    sqlx::Error::RowNotFound => Error::NotFound { id: request.id },
                    _ => common::internal_err!("db error: {err}"),
                })?;
                let mut epigram = None;
                if request.include_replies {
                    let Ref(gram) = cx
                        .epigram
                        .get_gram(epigram_api::gram::get::Request {
                            id: row.epigram_id.clone(),
                            include_replies: true,
                        })
                        .await
                        .map_err(|err| {
                            /* use epigram_api::gram::get::Error as GetGramError;
                            if err.is::<GetGramError>() {
                                match err.downcast::<GetGramError>().unwrap_or_log() {
                                    GetGramError::NotFound { id } => todo!(),
                                    GetGramError::Internal { message } => todo!(),
                                }
                            } */
                            common::internal_err!(
                                "err trying to get epigram from `epigram_api`: {err}"
                            )
                        })?;
                    epigram = Some(gram);
                }
                Post {
                    id: row.id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    epigram_id: row.epigram_id,
                    title: row.title,
                    url: row.url,
                    body: row.body,
                    author_username: row.author_username,
                    author_pub_key: row.author_pub_key,
                    author_pic_url: row.author_pic_url,
                    epigram,
                }
            }
        };
        Ok(out.into())
    }
}

impl From<&Error> for StatusCode {
    fn from(err: &Error) -> Self {
        use Error::*;
        match err {
            NotFound { .. } => Self::NOT_FOUND,
            // AccessDenied => Self::UNAUTHORIZED,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Deserialize, utoipa::IntoParams)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(default)]
    include_replies: bool,
}

impl HttpEndpoint for GetPost {
    type SharedCx = SharedContext;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/posts/:id";

    type HttpRequest = (Query<QueryParams>, Path<Uuid>, DiscardBody);

    fn request(
        (Query(params), Path(id), _): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request {
            /*auth_token, */ id,
            include_replies: params.include_replies,
            // include_replies: true,
        })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for GetPost {
    const TAG: &'static Tag = &crate::post::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [Post {
            id: default(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            epigram_id: "f26204069c8e8525502946fa9e7b9f51a1a3a9fb3bbd1263bf6fdc39af8572d61".into(),
            title: "Earth 2 reported to begin operations next circumsolar year".into(),
            url: Some("ipns://𝕏.com/stella_oort/48723494675897134423".into()),
            body: Some("Please sign in to see this xeet.".into()),
            author_username: "tazental".into(),
            author_pic_url: None,
            author_pub_key: "f196b70071ff6d9c6480677814ac78d2d1478a05a46c60d1dcd7afd21befb0b89"
                .into(),
            epigram: None,
        }]
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Error>> {
        use Error::*;
        vec![
            // ("Access Denied", AccessDenied),
            ("Not Found", NotFound { id: default() }),
            (
                "Internal server error",
                Internal {
                    message: "internal server error".into(),
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    use crate::post::testing::*;

    macro_rules! get_post_integ {
        ($(
            $name:ident: {
                uri: $uri:expr,
                // auth_token: $auth_token:expr,
                status: $status:expr,
                $(check_json: $check_json:expr,)?
                $(extra_assertions: $extra_fn:expr,)?
            },
        )*) => {
            mod integ {
                use super::*;
                common::integration_table_tests! {
                    $(
                        $name: {
                            uri: $uri,
                            method: "GET",
                            status: $status,
                            router: crate::post::router(),
                            cx_fn: crate::utils::testing::cx_fn_with_epigram,
                            $(check_json: $check_json,)?
                            // auth_token: $auth_token,
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    get_post_integ! {
        works_includes_replies: {
            uri: format!("/posts/{POST_01_ID}?includeReplies=true"),
            // auth_token: SERVICE.into(),
            status: StatusCode::OK,
            // check_json: serde_json::json!(*post_01).remove_keys_from_obj(&["createdAt"]),
            extra_assertions: &|EAArgs { response_json, .. }| {
                Box::pin(async move {
                    let resp_body_json = response_json.unwrap();
                    assert!(dbg!(resp_body_json)["epigram"].is_object());
                })
            },
        },
        works_excludes_replies: {
            uri: format!("/posts/{POST_01_ID}"),
            // auth_token: SERVICE.into(),
            status: StatusCode::OK,
            // check_json: serde_json::json!(*post_01).remove_keys_from_obj(&["createdAt"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let resp_body_json = response_json.unwrap();
                    assert!(resp_body_json["epigram"].is_null());
                })
            },
        },
        fails_if_not_found: {
            uri: format!("/posts/{}", Uuid::new_v4()),
            status: StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound",
            }),
        },
    }
}
