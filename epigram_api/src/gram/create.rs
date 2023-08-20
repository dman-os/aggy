use crate::interlude::*;

use super::Gram;

#[derive(Debug, Clone)]
pub struct CreateGram;

#[derive(Debug, Serialize, Deserialize, Validate, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Request {
    #[schema(min_length = 1)]
    #[validate(length(min = 1))]
    pub content: String,
    #[validate(length(min = 1), contains(pattern = "/"))] // FIXME: proper coty validation
    pub coty: String,
    pub parent_id: Option<String>,
    pub author_pubkey: String,
    #[serde(with = "common::codecs::sane_iso8601")]
    pub created_at: OffsetDateTime,
    pub id: String,
    pub sig: String,
    #[validate(length(min = 1))]
    pub author_alias: Option<String>,
    // pub author_notif_email: Option<String>,
}

fn validate_request(
    req: &Request,
) -> Result<
    (
        Vec<u8>,
        ed25519_dalek::VerifyingKey,
        ed25519_dalek::Signature,
    ),
    validator::ValidationErrors,
> {
    validator::Validate::validate(&req)?;
    let diff = OffsetDateTime::now_utc() - req.created_at;
    if !(diff.as_seconds_f64() < 60.0 && diff.as_seconds_f64() >= 0.0) {
        let mut issues = validator::ValidationErrors::new();
        issues.add(
            "createdAt",
            validator::ValidationError {
                code: Cow::Borrowed("created_too_long_ago"),
                message: Some(Cow::Borrowed(
                    "Submitted epigrams are expected to have been authored and signed less than a minute ago.",
                )),
                params: [(
                    std::borrow::Cow::from("value"),
                    serde_json::json!(req.created_at),
                )]
                .into_iter()
                .collect(),
            },
        );
        return Err(issues);
    }

    let id = crate::utils::id_for_gram(
        req.author_pubkey.as_str(),
        req.created_at,
        req.content.as_str(),
        req.coty.as_str(),
        req.parent_id.as_deref(),
    );

    let id_bytes = match common::utils::decode_hex_multibase(&req.id) {
        Ok(value) if &value[..] == &id.as_bytes()[..] => value,
        _ => {
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                "id",
                validator::ValidationError {
                    code: Cow::Borrowed("invalid_id"),
                    message: Some(Cow::Borrowed(
                        "Unable to decode id. Expecting a blake3 hash of the gram's contents encoded in multibase.",
                    )),
                    params: [(
                        std::borrow::Cow::from("value"),
                        serde_json::json!(req.id),
                    )]
                    .into_iter()
                    .collect(),
                },
            );
            return Err(issues);
        }
    };
    let pubkey = match common::utils::decode_hex_multibase(&req.author_pubkey).and_then(|buf| {
        ed25519_dalek::VerifyingKey::from_bytes(
            &buf[..]
                .try_into()
                .map_err(|err| eyre::eyre!("error converting slice to array: {err}"))?,
        )
        .map_err(|err| eyre::eyre!("error converting bytes to key: {err}"))
    }) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                "authorPubkey",
                validator::ValidationError {
                code: Cow::Borrowed("invalid_pubkey"),
                message: Some(Cow::Borrowed(
                    "Unable to decode pubkey. Expecting a ed25519 pubkey encoded using multibase.",
                )),
                params: [(
                    std::borrow::Cow::from("value"),
                    serde_json::json!(req.author_pubkey),
                )]
                .into_iter()
                .collect(),
                },
            );
            return Err(issues);
        }
    };

    let sig = match common::utils::decode_hex_multibase(&req.sig).and_then(|buf| {
        ed25519_dalek::Signature::from_slice(&buf[..])
            .map_err(|err| eyre::eyre!("error converting bytes to signature: {err}"))
    }) {
        Ok(value) if pubkey.verify_strict(&id_bytes[..], &value).is_ok() => value,
        _ => {
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                "sig",
                validator::ValidationError {
                    code: Cow::Borrowed("invalid_sig"),
                    message: Some(Cow::Borrowed(
                        "Provided sig was invalid. Expecting ed25519 hash of the id.",
                    )),
                    params: [(std::borrow::Cow::from("value"), serde_json::json!(req.sig))]
                        .into_iter()
                        .collect(),
                },
            );
            return Err(issues);
        }
    };
    Ok((id_bytes, pubkey, sig))
}

pub type Response = Ref<Gram>;

#[derive(Debug, Serialize, thiserror::Error, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum Error {
    #[error("parent not found at id {id:?}")]
    ParentNotFound { id: String },
    #[error("invalid input: {issues:?}")]
    InvalidInput {
        #[from]
        issues: ValidationErrors,
    },
    #[error("internal server error: {message:?}")]
    Internal { message: String },
}

#[async_trait::async_trait]
impl Endpoint for CreateGram {
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
        let (id_bytes, pubkey, sig) = validate_request(&request).map_err(ValidationErrors::from)?;
        let parent_id = match &request.parent_id {
            Some(parent_id) => Some(common::utils::decode_hex_multibase(&parent_id[..]).map_err(
                |_| Error::ParentNotFound {
                    id: request.id.clone(),
                },
            )?),
            None => None,
        };

        let out: Gram = match &cx.db {
            crate::Db::Pg { db_pool } => {
                let row = sqlx::query!(
                    r#"
WITH gram as (
    INSERT INTO grams.grams (
        id
        ,created_at
        ,content
        ,coty
        ,parent_id
        ,sig
        ,author_pubkey
        ,author_alias
        ,author_notif_email
    ) 
    VALUES (
        $1
        ,$2
        ,$3
        ,$4
        ,$5
        ,$6
        ,$7
        ,$8
        ,NULL
    ) RETURNING *
) SELECT 
    util.multibase_encode_hex(id) as "id!"
    ,created_at
    ,content
    ,coty
    ,util.multibase_encode_hex(parent_id) as "parent_id?"
    ,util.multibase_encode_hex(sig) as "sig!"
    ,util.multibase_encode_hex(author_pubkey) as "author_pubkey!"
    ,author_alias as "author_alias?"
FROM gram
"#,
                    &id_bytes,
                    &request.created_at,
                    &request.content,
                    &request.coty,
                    parent_id.as_ref(),
                    &sig.to_bytes()[..],
                    pubkey.as_bytes(),
                    request.author_alias.as_ref(),
                )
                .fetch_one(db_pool)
                .await
                .map_err(|err| match &err {
                    sqlx::Error::Database(boxed) if boxed.constraint().is_some() => {
                        match boxed.constraint().unwrap() {
                            "grams_parent_id_fkey" => Error::ParentNotFound {
                                id: request.parent_id.unwrap(),
                            },
                            _ => Error::Internal {
                                message: format!("db error: {err}"),
                            },
                        }
                    }
                    _ => Error::Internal {
                        message: format!("db error: {err}"),
                    },
                })?;
                Gram {
                    id: row.id,
                    created_at: row.created_at,
                    content: row.content,
                    coty: row.coty,
                    parent_id: row.parent_id,
                    author_pubkey: row.author_pubkey,
                    author_alias: row.author_alias,
                    sig: row.sig,
                    replies: default(),
                    reply_count: Some(0),
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
            ParentNotFound { .. } => Self::NOT_FOUND,
            InvalidInput { .. } => Self::BAD_REQUEST,
            Internal { .. } => Self::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HttpEndpoint for CreateGram {
    const METHOD: Method = Method::Post;
    const PATH: &'static str = "/grams";
    const SUCCESS_CODE: StatusCode = StatusCode::CREATED;

    type SharedCx = SharedContext;
    type HttpRequest = (Json<Request>,);

    fn request((Json(req),): Self::HttpRequest) -> Result<Self::Request, Self::Error> {
        Ok(req)
    }

    fn response(Ref(resp): Self::Response) -> HttpResponse {
        Json(resp).into_response()
    }
}

impl DocumentedEndpoint for CreateGram {
    const TAG: &'static Tag = &super::TAG;

    fn success_examples() -> Vec<serde_json::Value> {
        use crate::gram::testing::*;
        [GRAM_01.clone()]
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn errors() -> Vec<ErrorResponse<Self::Error>> {
        use crate::gram::testing::*;
        vec![
            (
                "Parent Not Found",
                Error::ParentNotFound {
                    id: GRAM_01_ID.into(),
                },
            ),
            (
                "Invalid input",
                Error::InvalidInput {
                    issues: {
                        let mut issues = validator::ValidationErrors::new();
                        issues.add(
                            "email",
                            validator::ValidationError {
                                code: std::borrow::Cow::from("email"),
                                message: None,
                                params: [(
                                    std::borrow::Cow::from("value"),
                                    serde_json::json!("bad.email.com"),
                                )]
                                .into_iter()
                                .collect(),
                            },
                        );
                        issues.into()
                    },
                },
            ),
            (
                "Internal server error",
                Error::Internal {
                    message: "internal server error".to_string(),
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    use super::Request;
    use crate::gram::testing::*;

    use ed25519_dalek::Signer;

    const TEST_PRIVKEY: &str = "f48cf7ffde6b73a4f5bc2749a335585d9750af7afc711063d85a104dc6c374e24";
    const TEST_PUBKEY: &str = "faecf2e46a2ae333aaf1a1ae8624d422bfcb57480ae25214b16bc12f03f32ff3e";

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fix_id_and_sig(request: Request, privkey: &str) -> Request {
        let prikey = common::utils::decode_hex_multibase(privkey).unwrap();
        let prikey = ed25519_dalek::SigningKey::from_bytes(&prikey[..].try_into().unwrap());
        let json = serde_json::to_string(&serde_json::json!([
            0,
            request.author_pubkey,
            request.created_at.unix_timestamp(),
            request.content,
            request.coty,
            request.parent_id,
        ]))
        .unwrap();
        let id = blake3::hash(json.as_bytes());
        let sig = common::utils::encode_hex_multibase(prikey.sign(id.as_bytes()).to_bytes());
        let id = common::utils::encode_hex_multibase(id.as_bytes());
        Request { id, sig, ..request }
    }

    fn fixture_request_json() -> serde_json::Value {
        let content = "The stars are a burning sun";
        let coty = "text/plain";
        let prikey = common::utils::decode_hex_multibase(TEST_PRIVKEY).unwrap();
        let prikey = ed25519_dalek::SigningKey::from_bytes(&prikey[..].try_into().unwrap());
        let author_pubkey = TEST_PUBKEY;
        let author_alias = "bridget";
        // let created_at = OffsetDateTime::from_unix_timestamp(1_690_962_268).unwrap();
        let created_at = OffsetDateTime::now_utc();
        let parent_id = GRAM_01_ID;
        let json = serde_json::to_string(&serde_json::json!([
            0,
            author_pubkey,
            created_at.unix_timestamp(),
            content,
            coty,
            parent_id,
        ]))
        .unwrap();
        let id = blake3::hash(json.as_bytes());
        let sig = common::utils::encode_hex_multibase(prikey.sign(id.as_bytes()).to_bytes());
        let id = common::utils::encode_hex_multibase(id.as_bytes());
        let created_at = created_at
            .format(&common::codecs::sane_iso8601::FORMAT)
            .unwrap();
        serde_json::json!({
            "content": content,
            "coty": coty,
            "createdAt": created_at,
            "authorPubkey": author_pubkey,
            "parentId": parent_id,
            "id": id,
            "sig": sig,
            "authorAlias": author_alias,
        })
    }

    common::table_tests! {
        validate,
        (request, err_field),
        {
            match crate::gram::create::validate_request(&request) {
                Ok(_) => {
                    if let Some(err_field) = err_field {
                        panic!("validation succeeded, was expecting err on field: {err_field}");
                    }
                }
                Err(err) => {
                    let err_field = err_field.expect("unexpected validation failure");
                    if !err.field_errors().contains_key(&err_field) {
                        panic!("validation didn't fail on expected field: {err_field}, {err:?}");
                    }
                }
            }
        }
    }

    validate! {
        helper_fix_id_and_sig_works: (
            fix_id_and_sig(
                Request {
                    content: "Shusha shumba".into(),
                    created_at: OffsetDateTime::now_utc(),
                    ..fixture_request()
                },
                TEST_PRIVKEY
            ),
            Option::<&str>::None,
        ),
        rejects_bad_id_created_at: (
            Request {
                created_at: OffsetDateTime::now_utc() - std::time::Duration::new(4, 0),
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_bad_id_content: (
            Request {
                content: "my name is not jeff".into(),
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_bad_id_coty: (
            Request {
                coty: "application/octet-stream".into(),
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_bad_id_parent_id: (
            Request {
                parent_id: Some(GRAM_02_ID.into()),
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_bad_id_author_pubkey: (
            Request {
                author_pubkey: GRAM_01.author_pubkey.clone(),
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_empty_content: (
            Request {
                content: "".into(),
                ..fixture_request()
            },
            Some("content"),
        ),
        rejects_invalid_coty: (
            fix_id_and_sig(
                Request {
                coty: "INVALID".into(),
                    ..fixture_request()
                },
                TEST_PRIVKEY
            ),
            Some("coty"),
        ),
        rejects_non_recent_timestamp: (
            fix_id_and_sig(
                Request {
                    created_at: OffsetDateTime::from_unix_timestamp(1_690_962_268).unwrap(),
                    ..fixture_request()
                },
                TEST_PRIVKEY
            ),
            Some("createdAt"),
        ),
        rejects_bad_sig: (
            Request {
                sig: GRAM_01.sig.clone(),
                ..fixture_request()
            },
            Some("sig"),
        ),
        rejects_empty_author_alias: (
            Request {
                author_alias: Some("".into()),
                ..fixture_request()
            },
            Some("author_alias"),
        ),
    }

    macro_rules! integ {
        ($(
            $name:ident: {
                status: $status:expr,
                body: $json_body:expr,
                $(check_json: $check_json:expr,)?
                $(extra_assertions: $extra_fn:expr,)?
            },
        )*) => {
            mod integ {
                use super::*;
                common::integration_table_tests! {
                    $(
                        $name: {
                            uri: "/grams",
                            method: "POST",
                            status: $status,
                            router: crate::gram::router(),
                            cx_fn: crate::utils::testing::cx_fn,
                            body: $json_body,
                            $(check_json: $check_json,)?
                            $(extra_assertions: $extra_fn,)?
                        },
                    )*
                }
            }
        };
    }

    integ! {
        works: {
            status: http::StatusCode::CREATED,
            body: fixture_request_json(),
            check_json: fixture_request_json().remove_keys_from_obj(&["createdAt","id","sig"]),
            extra_assertions: &|EAArgs { test_cx, response_json, .. }| {
                Box::pin(async move {
                    let cx = state_fn(test_cx);
                    let req_body_json = fixture_request_json();
                    let resp_body_json = response_json.unwrap();

                    let app = crate::gram::router().with_state(cx);
                    let resp = app
                        .oneshot(
                            http::Request::builder()
                                .method("GET")
                                .uri(format!("/grams/{}", resp_body_json["id"].as_str().unwrap()))
                                .body(Default::default())
                                .unwrap_or_log(),
                        )
                        .await
                        .unwrap_or_log();
                    assert_eq!(resp.status(), http::StatusCode::OK);
                    let body = resp.into_body();
                    let body = hyper::body::to_bytes(body).await.unwrap_or_log();
                    let body = serde_json::from_slice(&body).unwrap_or_log();
                    tracing::info!(?body, "test");
                    check_json(
                        ("expected", &req_body_json.remove_keys_from_obj(&["createdAt", "id", "sig"])),
                        ("response", &body),
                    );
                })
            },
        },
        author_alias_is_optional: {
            status: http::StatusCode::CREATED,
            body: fixture_request_json().remove_keys_from_obj(&["authorAlias"]),
            check_json: fixture_request_json().remove_keys_from_obj(&["authorAlias", "createdAt", "id", "sig"]),
        },
        fails_if_parent_id_not_found: {
            status: http::StatusCode::NOT_FOUND,
            body: serde_json::json!(fix_id_and_sig(
                Request {
                    parent_id: Some("f0eb8a1906dc580b7afdb55db82d2bf384aa4512e9d07d87fb5ca3e6b0cf7cf21".into()),
                    ..fixture_request()
                },
                TEST_PRIVKEY
            )),
            check_json: serde_json::json!({
                "error": "parentNotFound"
            }),
        },
        /*
        // FIXME:
        fails_if_email_occupied: {
            status: http::StatusCode::BAD_REQUEST,
            body: fixture_request_json().destructure_into_self(
                serde_json::json!({ "email": USER_01_EMAIL })
            ),
            check_json: serde_json::json!({
                "error": "emailOccupied"
            }),
        },*/
    }
}
