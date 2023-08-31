use crate::interlude::*;

use crate::event::Event;

#[derive(Debug, Clone)]
pub struct CreateEvent;

pub type Request = Event;

fn validate_request(
    req: &Request,
) -> Result<
    (
        [u8; 32],
        k256::schnorr::VerifyingKey,
        k256::schnorr::Signature,
    ),
    validator::ValidationErrors,
> {
    // validator::Validate::validate(&req)?;
    /* let diff = OffsetDateTime::now_utc() - req.created_at;
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
    }*/

    let (id_bytes, json_bytes) = crate::event::id_for_event(
        req.pubkey.as_str(),
        req.created_at,
        req.kind,
        &req.tags,
        req.content.as_str(),
    );

    match data_encoding::HEXLOWER.decode(req.id.as_bytes()) {
        Ok(value) if value[..] == id_bytes[..] => {}
        _ => {
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                "id",
                validator::ValidationError {
                    code: Cow::Borrowed("invalid_id"),
                    message: Some(Cow::Borrowed(
                        "Unable to decode id. Expecting a sha256 hash of the event's contents according to NIP-01.",
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
    let pubkey = match data_encoding::HEXLOWER
        .decode(req.pubkey.as_bytes())
        .map_err(|err| eyre::eyre!("error decoding hex pubkey: {err}"))
        .and_then(|buf| {
            k256::schnorr::VerifyingKey::from_bytes(&buf[..])
                .map_err(|err| eyre::eyre!("error converting bytes to key: {err}"))
        }) {
        Ok(pubkey) => pubkey,
        Err(_) => {
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                "pubkey",
                validator::ValidationError {
                code: Cow::Borrowed("invalid_pubkey"),
                message: Some(Cow::Borrowed(
                    "Unable to decode pubkey. Expecting a secp256k1 pubkey encoded in lowercase hex.",
                )),
                params: [(
                    std::borrow::Cow::from("value"),
                    serde_json::json!(req.pubkey),
                )]
                .into_iter()
                .collect(),
                },
            );
            return Err(issues);
        }
    };

    use k256::schnorr::signature::*;
    let sig = match data_encoding::HEXLOWER_PERMISSIVE
        .decode(req.sig.as_bytes())
        .map_err(|err| eyre::eyre!("error decoding hex sig: {err}"))
        .and_then(|buf| {
            k256::schnorr::Signature::try_from(&buf[..])
                .map_err(|err| eyre::eyre!("error converting bytes to signature: {err}"))
        }) {
        Ok(value) if pubkey.verify(&json_bytes[..], &value).is_ok() => value,
        err => {
            error!(?err, "sig fail");
            let mut issues = validator::ValidationErrors::new();
            issues.add(
                "sig",
                validator::ValidationError {
                    code: Cow::Borrowed("invalid_sig"),
                    message: Some(Cow::Borrowed(
                        "Provided sig was invalid. Expecting schnorr k256 hash of the id.",
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

pub struct Response {
    id: String,
}
impl Response {
    pub fn to_nostr_ok(self) -> serde_json::Value {
        let Self { id } = self;
        json!(["OK", id, true,])
    }
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(crate = "serde", rename_all = "camelCase")]
#[error("error processing event {event_id}: {kind}")]
pub struct Error {
    event_id: String,
    #[source]
    kind: ErrorKind,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(crate = "serde", rename_all = "camelCase", tag = "error")]
pub enum ErrorKind {
    #[error("duplicate: event already recieved")]
    Duplicate,
    #[error("invalid:{issues}")]
    InvalidInput {
        #[from]
        issues: ValidationErrors,
    },
    #[error("error:internal: {message}")]
    Internal { message: String },
}

impl Error {
    pub fn to_nostr_ok(self) -> serde_json::Value {
        let Self { event_id, kind } = self;
        json!(["OK", event_id, false, format!("{kind}")])
    }
}

#[async_trait::async_trait]
impl Endpoint for CreateEvent {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Cx = Context;

    #[tracing::instrument(skip(cx), err)]
    async fn handle(
        &self,
        cx: &Self::Cx,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let (id_bytes, pubkey, sig) = validate_request(&request)
            .map_err(ValidationErrors::from)
            .map_err(|kind| Error {
                event_id: request.id.clone(),
                kind: kind.into(),
            })?;
        match &cx.db {
            crate::Db::Pg { db_pool } => {
                sqlx::query!(
                    r#"
INSERT INTO public.events (
    id
    ,pubkey
    ,created_at
    ,kind
    ,tags
    ,content
    ,sig
) VALUES
(
    $1 
    ,$2
    ,$3
    ,$4
    ,$5
    ,$6
    ,$7
)
                "#,
                    &id_bytes[..],
                    &pubkey.to_bytes()[..],
                    request.created_at,
                    request.kind as i32,
                    &json!(request.tags),
                    request.content.as_str(),
                    &sig.to_bytes()[..],
                )
                .execute(db_pool)
                .await
                .map_err(|err| {
                    if let sqlx::Error::Database(boxed) = &err {
                        if let Some("events_pkey") = boxed.constraint() {
                            return ErrorKind::Duplicate;
                        }
                    }
                    panic!("db error: {err}");
                })
                .map_err(|kind| Error {
                    event_id: request.id.clone(),
                    kind,
                })?;
            }
        }
        crate::connect::pub_event(cx, &request)
            .await
            .unwrap_or_log();
        Ok(Response { id: request.id })
    }
}

#[cfg(test)]
mod tests {
    use crate::interlude::*;

    use super::Request;

    use crate::event::testing::*;
    const TEST_PRIVKEY: &str = "95dfc6261ec6c66b3ec68e1b019cf6420e1d676c29c1241ec5dea551ed89e338";

    fn fixture_request() -> Request {
        serde_json::from_value(fixture_request_json()).unwrap()
    }

    fn fix_id_and_sig(request: Request, privkey: &str) -> Request {
        let privkey = data_encoding::HEXLOWER.decode(privkey.as_bytes()).unwrap();
        let privkey = k256::schnorr::SigningKey::from_bytes(&privkey[..]).unwrap();
        let (id, sig) = crate::event::hex_id_and_sig_for_event(
            &privkey,
            request.pubkey.as_str(),
            request.created_at,
            request.kind,
            &request.tags,
            request.content.as_str(),
        );
        Request { id, sig, ..request }
    }

    fn fixture_request_json() -> serde_json::Value {
        let content = "The stars are a burning sun";

        let prikey = TEST_PRIVKEY;
        let prikey = data_encoding::HEXLOWER.decode(prikey.as_bytes()).unwrap();
        let prikey = k256::schnorr::SigningKey::from_bytes(&prikey[..]).unwrap();

        let pubkey = prikey.verifying_key().to_bytes();
        let pubkey = data_encoding::HEXLOWER.encode(&pubkey[..]);

        // let created_at = OffsetDateTime::from_unix_timestamp(1_690_962_268).unwrap();
        let created_at = OffsetDateTime::now_utc();

        let tags = vec![
            vec!["author".to_string(), "bridget".to_string()],
            vec!["e".to_string(), EVENT_01_ID.to_string()],
        ];

        let kind = 1;
        let (id, sig) = crate::event::hex_id_and_sig_for_event(
            &prikey,
            &pubkey[..],
            created_at,
            kind,
            &tags,
            content,
        );
        serde_json::json!({
            "id": id,
            "pubkey": pubkey,
            "created_at": created_at.unix_timestamp(),
            "kind": kind,
            "tags": tags,
            "content": content,
            "sig": sig,
        })
    }

    common::table_tests! {
        validate,
        (request, err_field),
        {
            common::utils::testing::setup_tracing_once();
            match crate::event::create::validate_request(&request) {
                Ok(_) => {
                    if let Some(err_field) = err_field {
                        panic!("validation succeeded, was expecting err on field: {err_field}");
                    }
                }
                Err(err) => {
                    let err_field = err_field.expect_or_log("unexpected validation failure");
                    if !err.field_errors().contains_key(&err_field) {
                        panic!("validation didn't fail on expected field: {err_field}, {err:?}");
                    }
                }
            }
        }
    }

    validate! {
        is_upto_spec: (
            serde_json::from_str(r#"{"id":"4d84fae57fa93c836f161e75e404f6e489fb6c9737cc18cc0f757b7f3cacbaa6","pubkey":"b021c176157909a4515e3a182d92c17c28c62c9304974d944e49da562888a4b0","created_at":1642760731,"kind":2,"tags":[],"content":"wss://rsslay.fiatjaf.com","sig":"bdc8f2a7a731328bb002dae805ff1b21b1a175e48693e4d2c8fbc97f50fff506cd0f0dda8f6792b3d0ded88219211ed3161a36e86827c06ceb7becdba2008977"}"#)
                .unwrap(),
            Option::<&str>::None,
        ),
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
        rejects_bad_id_kind: (
            Request {
                kind: 12,
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_bad_id_tags: (
            Request {
                tags: vec![
                    vec!["alt".to_string(), "reply".to_string(), "wss://nostr.example".to_string()]
                ],
                ..fixture_request()
            },
            Some("id"),
        ),
        rejects_bad_id_pubkey: (
            Request {
                pubkey: {
                    let keypair = k256::schnorr::SigningKey::random(&mut rand::thread_rng());
                    let pubkey = keypair.verifying_key();
                    data_encoding::HEXLOWER.encode(&pubkey.to_bytes())
                },
                ..fixture_request()
            },
            Some("id"),
        ),
        /* rejects_non_recent_timestamp: (
            fix_id_and_sig(
                Request {
                    created_at: OffsetDateTime::from_unix_timestamp(1_690_962_268).unwrap(),
                    ..fixture_request()
                },
                TEST_PRIVKEY
            ),
            Some("createdAt"),
        ), */
        rejects_bad_sig: (
            Request {
                sig: EVENT_01.sig.clone(),
                ..fixture_request()
            },
            Some("sig"),
        ),
    }

    common::table_tests! {
        integ tokio,
        (request_json, expected_json),
        {
            let (mut testing, cx) = crate::utils::testing::cx_fn(common::function_full!()).await;
            {
                let event = serde_json::from_value(request_json).unwrap();
                let ok = match crate::event::create::CreateEvent.handle(&cx, event).await{
                    Ok(value) => value.to_nostr_ok(),
                    Err(value) => value.to_nostr_ok(),
                };
                tracing::info!(?ok);
                check_json(
                    ("expected", &expected_json),
                    ("response", &ok),
                );
            }
            testing.close().await;
        },
        multi_thread: true,
    }

    integ! {
        works: (
            fixture_request_json(),
            serde_json::json!([
                "OK", fixture_request().id, true
            ])
        ),
        rejects_duplicates: (
            json!(*EVENT_01),
            serde_json::json!([
                "OK", EVENT_01_ID, false,
            ])
        ),
    }
}