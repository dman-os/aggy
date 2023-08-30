use crate::interlude::*;

use std::collections::HashMap;

use deps::redis::FromRedisValue;
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
impl redis::ToRedisArgs for Event {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let buf = serde_json::to_vec(self).unwrap_or_log();
        out.write_arg(&buf[..]);
    }
}
impl FromRedisValue for Event {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Data(vec) => Ok(serde_json::from_slice(&vec[..])?),
            _ => Err((
                redis::ErrorKind::TypeError,
                "unexpected redis Value for Event",
            )
                .into()),
        }
        /* let map: ahash::AHashMap<String, &redis::Value> = FromRedisValue::from_redis_value(v)?;
        Ok(Self {
            id: {
                let id = map
                    .get("id")
                    .ok_or_else(|| eyre::eyre!("id not found in map for Event"))?;
                FromRedisValue::from_redis_value(*id)?
            },
            pubkey: {
                let pubkey = map
                    .get("pubkey")
                    .ok_or_else(|| eyre::eyre!("pubkey not found in map for Event"))?;
                FromRedisValue::from_redis_value(*pubkey)?
            },
            created_at: {
                let created_at = map
                    .get("created_at")
                    .ok_or_else(|| eyre::eyre!("created_at not found in map for Event"))?;
                let created_at: i64 = FromRedisValue::from_redis_value(*created_at)?;
                OffsetDateTime::from_unix_timestamp(created_at)
                    .map_err(|err| eyre::eyre!("invalid timestamp found in map for Event: {err}"))?
            },
            kind: {
                let kind = map
                    .get("kind")
                    .ok_or_else(|| eyre::eyre!("kind not found in map for Event"))?;
                let kind: i64 = FromRedisValue::from_redis_value(*created_at)?;
                u16::try_from(kind)
                    .map_err(|err| eyre::eyre!("invalid kind found in map for Event: {err}"))?
            },
            tags: {
                let tags = map
                    .get("tags")
                    .ok_or_else(|| eyre::eyre!("kind not found in map for Event"))?;
                let tags: String = FromRedisValue::from_redis_value(*tags);
                serde_json::from_str(tags.as_str()).map_err(|err| {
                    eyre::eyre!("error decoding tags json string for Event: {err}")
                })?
            },
            content: {
                let content = map
                    .get("content")
                    .ok_or_else(|| eyre::eyre!("content not found in map for Event"))?;
                FromRedisValue::from_redis_value(*content)?
            },
            sig: {
                let sig = map
                    .get("sig")
                    .ok_or_else(|| eyre::eyre!("sig not found in map for Event"))?;
                FromRedisValue::from_redis_value(*sig)?
            },
        }) */
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

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.ids.is_none()
            && self.authors.is_none()
            && self.kinds.is_none()
            && self.since.is_none()
            && self.until.is_none()
            && self.tags.is_none()
    }
    // TODO: test suite for this
    pub fn matches(&self, event: &Event) -> bool {
        [
            self.ids.as_deref().map(|list| list.contains(&event.id)),
            self.kinds.as_deref().map(|list| list.contains(&event.kind)),
            self.since.map(|ts| ts < event.created_at),
            self.until.map(|ts| ts > event.created_at),
            self.authors
                .as_deref()
                .map(|list| list.contains(&event.pubkey)),
            self.tags.as_ref().map(|map| {
                let mut match_ctr = 0;
                for tag in &event.tags {
                    // we're only interested in [key, value, ...] tags
                    if tag.len() < 2 {
                        continue;
                    }
                    let mut chars = tag[0][1..].chars();
                    // the key is an empty string
                    let Some(char) = chars.next() else {
                        continue;
                    };
                    // the key has more than one char, not eligible
                    if chars.next().is_some() {
                        continue;
                    }
                    // filter is not interested in this tag
                    let Some(list) = map.get(&char) else {
                        continue;
                    };
                    if list.contains(&tag[1]) {
                        match_ctr += 1;
                    }
                }
                match_ctr == map.len()
            }),
        ]
        .into_iter()
        .flatten()
        .all(|val| val)
    }
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // use a subset struct to decode most of the items
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
        // but use go through a json object step first to find all the tag filters
        let json: serde_json::Map<String, Value> = Deserialize::deserialize(deserializer)?;

        let mut tags: HashMap<char, Vec<String>> = default();
        for (key, val) in json.iter() {
            if let Some(tag) = key.strip_prefix('#') {
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
        // we decode Inner from the json
        let inner: Inner = serde_json::from_value(Value::Object(json))
            .map_err(|err| serde::de::Error::custom(format!("invalid filter: {err}")))?;
        Ok(Self {
            ids: inner.ids,
            kinds: inner.kinds,
            since: match inner.since {
                Some(ts) => Some(OffsetDateTime::from_unix_timestamp(ts).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(ts),
                        &"a valid utc unix timestamp",
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
            tags: if !tags.is_empty() { Some(tags) } else { None },
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
        "3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f";

    pub static EVENT_03: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_03_ID.into(),
        pubkey: "bd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815300).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[]"#).unwrap(),
        content: r#"I have information that'll lead to the arrest of Kermit The Frog"#.into(),
        sig: "f7d63de3be8c33334363098e53507cd5dc211d73e9e57be254c5b1035718cc4678501bc1a18d1afabe9b9b35968ba41440cae5bf46db3ebeb3ab1cf1eb359fd9".into(),
    }
    });

    pub const EVENT_04_ID: &str =
        "6b4c4c5818219aca0055f38c1dc255907f5fbcf21b0332857cfddf697ac91cd7";

    pub static EVENT_04: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_04_ID.into(),
        pubkey: "bd9002616950efb59b2c09446735b215b400052779ace5779f7d9c1290a8fa8e".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815400).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[["e","3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f"]]"#).unwrap(),
        content: r#"I'm glad people are paying attention. Information will be released soonTM. Meanwhile, I'll be selling Henson-gate tank-tops and jerseys. Links in my bio"#.into(),
        sig: "8a245231616d31b5ff13401884e798bf6947570c29093ac3e3850a72991c02e6deed77f60ca9921f4b4901648ed03a3e70fe5d59bc623d6c9afb824e270e60f5".into(),
    }
    });

    pub const EVENT_05_ID: &str =
        "e974080cde211594bbf3197ec9bceb43a27ed67366671fa69d5b65c1848d2f6e";

    pub static EVENT_05: Lazy<Event> = Lazy::new(|| {
        Event {
        id: EVENT_05_ID.into(),
        pubkey: "167c3b7d2640757b2d276c0f9c50d6820aa45208f97acd06a76920e532639c20".into(),
        created_at: OffsetDateTime::from_unix_timestamp(1692815500).unwrap(),
        kind: 1,
        tags: serde_json::from_str(r#"[["e","3d849f6890e511f8ffefdce494da6f95789e4b8a0354275c227b1aa81362b20f"]]"#).unwrap(),
        content: r#"Henson-gate"#.into(),
        sig: "f7a4e72838ca5062aadfb9d56bc837012374bdc4050f7aa004f3c81881e0d303a1ca6b45aa6f72555e7bda0f571d5572a8acd9b284ed5809d5f1ff652f06b0f3".into(),
    }
    });
}
