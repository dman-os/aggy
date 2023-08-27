use crate::interlude::*;

use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct Event {
    pub id: String,
    pub pubkey: String,
    #[serde(with = "time::serde::timestamp")]
    pub created_at: OffsetDateTime,
    // #[sqlx(try_from = "i64")]
    pub kind: u16,
    // #[sqlx(try_from = "serde_json::Value")]
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl FromRow<'_, sqlx::postgres::PgRow> for Event {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        let tags: Value = row.try_get("tags")?;
        let tags = match serde_json::from_value(tags) {
            Ok(val) => val,
            Err(err) => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "tags".into(),
                    source: err.into(),
                })
            }
        };
        let kind: i32 = row.try_get("kind")?;
        let kind = match kind.try_into() {
            Ok(val) => val,
            Err(err) => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "kind".into(),
                    source: eyre::eyre!("kind doesn't fit in u16: {err}").into(),
                })
            }
        };
        Ok(Self {
            id: row.try_get("id")?,
            pubkey: row.try_get("pubkey")?,
            created_at: row.try_get("created_at")?,
            content: row.try_get("content")?,
            sig: row.try_get("sig")?,
            kind,
            tags,
        })
    }
}

pub fn id_for_event(
    pubkey: &str,
    created_at: OffsetDateTime,
    kind: u16,
    tags: &Vec<Vec<String>>,
    content: &str,
) -> Vec<u8> {
    use k256::sha2::*;
    let json = serde_json::to_vec(&serde_json::json!([
        0,
        pubkey,
        created_at.unix_timestamp(),
        kind,
        tags,
        content,
    ]))
    .unwrap();
    let mut hasher = k256::sha2::Sha256::new();
    hasher.update(json);
    hasher.finalize().to_vec()
}

pub fn id_and_sig_for_event(
    privkey: &k256::schnorr::SigningKey,
    pubkey: &str,
    created_at: OffsetDateTime,
    kind: u16,
    tags: &Vec<Vec<String>>,
    content: &str,
) -> (Vec<u8>, k256::schnorr::Signature) {
    use k256::schnorr::signature::*;
    let id = crate::event::id_for_event(pubkey, created_at, kind, tags, content);
    let sig = privkey.sign(&id[..]);
    (id, sig)
}

pub fn hex_id_and_sig_for_event(
    privkey: &k256::schnorr::SigningKey,
    pubkey: &str,
    created_at: OffsetDateTime,
    kind: u16,
    tags: &Vec<Vec<String>>,
    content: &str,
) -> (String, String) {
    let (id, sig) = id_and_sig_for_event(privkey, pubkey, created_at, kind, tags, content);
    let id = data_encoding::HEXLOWER.encode(&id[..]);
    let sig = data_encoding::HEXLOWER.encode(&sig.to_bytes()[..]);
    (id, sig)
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub ids: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
    pub kinds: Option<Vec<u16>>,
    pub since: Option<OffsetDateTime>,
    pub until: Option<OffsetDateTime>,
    pub limit: Option<usize>,
    pub tags: Option<HashMap<char, Vec<String>>>,
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(crate = "serde")]
        pub struct Inner {
            pub ids: Option<Vec<String>>,
            pub authors: Option<Vec<String>>,
            pub kinds: Option<Vec<u16>>,
            // #[serde(with = "time::serde::timestamp::option")]
            pub since: Option<i64>,
            // #[serde(with = "time::serde::timestamp::option")]
            pub until: Option<i64>,
            pub limit: Option<usize>,
        }
        let json: serde_json::Map<String, Value> = Deserialize::deserialize(deserializer)?;
        let mut tags: HashMap<char, Vec<String>> = default();
        for (key, val) in json.iter() {
            if key.starts_with('#') {
                let tag = &key[1..];
                let mut iter = tag.chars();
                let Some(tag) = iter.next() else {
                    return Err(
                        serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(&key[..]),
                        &"a single char tag"
                        )
                    );
                };
                // we're only interested in single char tags
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(&key[..]),
                        &"a single char tag",
                    ));
                }
                // FIXME: unnecessary clone
                let val: Vec<String> = serde_json::from_value(val.clone()).map_err(|err| {
                    serde::de::Error::custom(format!("invalid tag filter: {key} -> {val} | {err}"))
                })?;
                tags.insert(tag, val);
            }
        }
        let inner: Inner = serde_json::from_value(Value::Object(json))
            .map_err(|err| serde::de::Error::custom(format!("invalid filter: {err}")))?;
        Ok(Self {
            ids: inner.ids,
            kinds: inner.kinds,
            since: match inner.since {
                Some(ts) => Some(OffsetDateTime::from_unix_timestamp(ts).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(ts),
                        &"a avlid utc unix timestamp",
                    )
                })?),
                None => None,
            },
            until: match inner.until {
                Some(ts) => Some(OffsetDateTime::from_unix_timestamp(ts).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(ts),
                        &"a valid utc unix timestamp",
                    )
                })?),
                None => None,
            },
            limit: inner.limit,
            authors: inner.authors,
            tags: Some(tags),
        })
    }
}

pub mod create;
pub mod list;

pub mod testing {
    use super::*;

    use once_cell::sync::Lazy;

    pub const EVENT_01_ID: &str =
        "b042eae42505d83996af3694f47224128596c89a3ea1a7fd27ea43c8e559cf20";

    pub static EVENT_01: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_01_ID.into(),
        pubkey: "7ecee90e906e56d7b20b2e76cdb83b786352d2bea53495e34ad556a989f7d39b".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815146).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[]"#).unwrap(),
        content: r#"the internet would be a better place if it was shut down on tuesdays or the like"#.into(),
        sig: "ff8925580d86f8cbf0de60eca4e1984e526bbf273801dde7824e2c1ee23e6ab70f41f929092c32440e40cfd45f4532dfc03be28b5fc271fd51825b7aafdd0104".into(),
    }
    });

    pub const EVENT_02_ID: &str =
        "1bb1d6acee88cd925c62e547e10c24ae65effe9286e4f1840e222643db76c833";

    pub static EVENT_02: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_02_ID.into(),
        pubkey: "a6dff3503ca65ecf97371f2ba3348c2385e01c0212d1317dcb3a6d843ff08949".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815146).unwrap(),
        kind: 0,
        tags: serde_json::from_str(r#"[["p","f72657e01156d2c9b251111e73d58236dfb7de5ca69e1b53f0a938528f16c265"]]"#).unwrap(),
        content: r#"{"about":"weaponized stink eye","name":"bridget","picture":"https://coro.na/virus.png"}"#.into(),
        sig: "3fd86ad14a171043b1ca9cacb58377bf8091288394f80e20ee30ed4e9adac7564045d2596eb3b95b6050ef8f2dd0df4a5b702f07d3ade44082934cce4fe869bb".into(),
    }
    });

    pub const EVENT_03_ID: &str =
        "8a8228471b5a0de4ec033749f90d2dbef1247b424bbb4c94b630575d84e782ce";

    pub static EVENT_03: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_03_ID.into(),
        pubkey: "bd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815146).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[]"#).unwrap(),
        content: r#"I have information that'll lead to the arrest of Kermit The Frog"#.into(),
        sig: "aa1b89e0f86dca2e930c57f2311dedcc30c9a2ff13f56dcfc0cf018c8f5e2d867bacd9889fafc5cd6e33acb8e8bc17de7655bb67f5813477cd0c9f0de0d5bfb8".into(),
    }
    });

    pub const EVENT_04_ID: &str =
        "ec41a05e3f5921d1b16b807f5c6e77b54349819fc59a998d341e8e15bda378e6";

    pub static EVENT_04: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_04_ID.into(),
        pubkey: "bd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815146).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[["e","8a8228471b5a0de4ec033749f90d2dbef1247b424bbb4c94b630575d84e782ce"]]"#).unwrap(),
        content: r#"I'm glad people are paying attention. Information will be released soonTM. Meanwhile, I'll be selling Henson-gate tank-tops and jerseys. Links in my bio"#.into(),
        sig: "fbeaf4ac101e8252e9a4cce13e8004b4232b5a9e1ec236e49312151f87b24025fea94c6f25171eb7e68b048fc9b44c0f5e443284f65ca9087f15af0cef6efcb3".into(),
    }
    });

    pub const EVENT_05_ID: &str =
        "51acf76b8a5676950bd8b40bff62ee652b7d672cb95d029a466185eb1291dc5a";

    pub static EVENT_05: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_05_ID.into(),
        pubkey: "167c3b7d2640757b2d276c0f9c50d6820aa45208f97acd06a76920e532639c20".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815146).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[["e","8a8228471b5a0de4ec033749f90d2dbef1247b424bbb4c94b630575d84e782ce"]]"#).unwrap(),
        content: r#"Henson-gate"#.into(),
        sig: "14ee5f9a5e2aa95064e297a4801b4b0c12392d601c67fd3a74b8dfebe2baa525b129eaa51d713531693ff18eaf365c14cb19a301c553629b9d7853950dc1bd55".into(),
    }
    });
}
