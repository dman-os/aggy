use crate::interlude::*;

use super::Gram;

#[derive(Clone, Copy, Debug)]
pub struct GetGram;

#[derive(Debug)]
pub struct Request {
    // pub auth_token: BearerToken,
    pub id: String,
}

pub type Response = Ref<Gram>;

#[derive(Debug, thiserror::Error, Serialize, ToSchema)]
#[serde(crate = "serde", tag = "error", rename_all = "camelCase")]
pub enum Error {
    #[error("gram not found at id: {id:?}")]
    NotFound { id: String },
    // #[error("{self:?}")]
    // AccessDenied,
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl Endpoint for GetGram {
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
        let id_byte = match (&request.id[0..1], hex::decode(&request.id[1..])) {
            ("f", Ok(id_byte)) => id_byte,
            _ => {
                return Err(Error::NotFound { id: request.id })
            }
        };

        match &cx.db {
            crate::Db::Pg { db_pool } => sqlx::query_as!(
                Gram,
                r#"
SELECT 
    util.multibase_encode_hex(id) as "id!"
    ,created_at
    ,content
    ,mime
    ,util.multibase_encode_hex(parent_id) as "parent_id?"
    ,util.multibase_encode_hex(sig) as "sig!"
    ,util.multibase_encode_hex(author_pubkey) as "author_pubkey!"
    ,author_alias as "author_alias?"
FROM grams.grams 
WHERE id = $1
                "#,
                &id_byte
            )
            .fetch_one(db_pool)
            .await
            .map(|val| val.into())
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => Error::NotFound { id: request.id },
                _ => Error::Internal {
                    #[cfg(not(censor_internal_errors))]
                    message: format!("db error: {err}"),
                    #[cfg(censor_internal_errors)]
                    message: format!("internal server error"),
                },
            }),
        }
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

impl HttpEndpoint for GetGram {
    type SharedCx = SharedContext;
    const METHOD: Method = Method::Get;
    const PATH: &'static str = "/grams/:id";

    type HttpRequest = (/*TypedHeader<BearerToken>,*/ Path<String>, DiscardBody);

    fn request(
        (/*TypedHeader(auth_token), */ Path(id), _): Self::HttpRequest,
    ) -> Result<Self::Request, Self::Error> {
        Ok(Request { /*auth_token, */ id })
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for GetGram {
    const TAG: &'static Tag = &crate::gram::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        [super::testing::GRAM_01.clone()]
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Error>> {
        use Error::*;
        vec![
            // ("Access Denied", AccessDenied),
            (
                "Not Found",
                NotFound {
                    id: "asldkfjaslkdfja".into(),
                },
            ),
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

    use crate::gram::testing::*;

    macro_rules! get_gram_integ {
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
                            router: crate::gram::router(),
                            state_fn: crate::utils::testing::state_fn,
                            $(check_json: $check_json,)?
                            // auth_token: $auth_token,
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    get_gram_integ! {
        works: {
            uri: format!("/grams/{GRAM_01_ID}"),
            // auth_token: SERVICE.into(),
            status: StatusCode::OK,
            check_json: serde_json::json!(*GRAM_01).remove_keys_from_obj(&["createdAt"]),
        },
        fails_if_not_found: {
            uri: format!("/grams/{}", Uuid::new_v4()),
            status: StatusCode::NOT_FOUND,
            check_json: serde_json::json!({
                "error": "notFound",
            }),
        },
    }
}
