use crate::interlude::*;

use serde::{Deserialize, Serialize};

pub const DEFAULT_LIST_LIMIT: usize = 25;

pub trait SortingField {
    fn sql_field_name(&self) -> String;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub enum SortingOrder {
    Ascending,
    Descending,
}

impl SortingOrder {
    #[inline]
    pub fn sql_key_word(&self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "serde", rename_all = "camelCase")]
pub struct Cursor<T, S>
where
    S: SortingField + Clone + Copy,
{
    pub value: T,
    pub field: S,
    pub order: SortingOrder,
    pub filter: Option<String>,
}

const CURSOR_VERSION: usize = 1;

impl<T, S> Cursor<T, S>
where
    S: Serialize + SortingField + Clone + Copy,
    T: Serialize,
{
    pub fn to_encoded_str(&self) -> String {
        use std::fmt::Write;
        // let mut out = format!("{CURSOR_VERSION}:");
        let mut out = String::new();
        {
            std::write!(&mut out, "{CURSOR_VERSION}:").unwrap_or_log();
            let mut compressed = Vec::new();
            // let mut b64_w = base64::write::EncoderWriter::new(&mut out, &base64::prelude::BASE64_URL_SAFE);
            {
                let mut brotli_w = brotli::CompressorWriter::new(&mut compressed, 4096, 5, 21);
                serde_json::to_writer(&mut brotli_w, &self).unwrap_or_log();
            }
            data_encoding::BASE64URL.encode_append(&compressed, &mut out);
        }
        out
    }
}

impl<T, S> std::str::FromStr for Cursor<T, S>
where
    T: serde::de::DeserializeOwned,
    S: SortingField + Clone + Copy + serde::de::DeserializeOwned,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ver_str, payload_str) = s.split_once(':').ok_or(())?;
        let version: usize = ver_str.parse().map_err(|_| ())?;
        if version != CURSOR_VERSION {
            return Err(());
        }
        let compressed = data_encoding::BASE64URL
            .decode(payload_str.as_bytes())
            .map_err(|_| ())?;
        let mut cursor = std::io::Cursor::new(&compressed);
        let mut json = Vec::new();
        brotli::BrotliDecompress(&mut cursor, &mut json).map_err(|_| ())?;
        tracing::info!("{}", std::str::from_utf8(&json).unwrap_or_log());
        serde_json::from_slice(&json[..]).map_err(|_| ())
    }
}

pub fn validate_list_req<S>(
    after_cursor: Option<&str>,
    before_cursor: Option<&str>,
    filter: Option<&str>,
    sorting_field: Option<S>,
    sorting_order: Option<SortingOrder>,
) -> Result<(), validator::ValidationError>
where
    S: SortingField + Clone + Copy + Serialize + utoipa::ToSchema<'static>,
{
    match (before_cursor.as_ref(), after_cursor.as_ref()) {
        (Some(before_cursor), Some(after_cursor)) => Err(validator::ValidationError {
            code: "before_and_after_cursors_at_once".into(),
            message: Some("both beforeCursor and afterCursor are present".into()),
            params: [
                (
                    "beforeCursor".into(),
                    serde_json::json!({ "value": before_cursor }),
                ),
                (
                    "afterCursor".into(),
                    serde_json::json!({ "value": after_cursor }),
                ),
            ]
            .into_iter()
            .collect(),
        }),
        (None, Some(cursor)) | (Some(cursor), None)
            if sorting_field.is_some() || sorting_order.is_some() || filter.is_some() =>
        {
            Err(validator::ValidationError {
                code: "both_cursor_and_sorting_or_filter".into(),
                message: Some("both beforeCursor and afterCursor are present".into()),
                params: [
                    Some((
                        if after_cursor.is_some() {
                            "afterCursor".into()
                        } else {
                            "beforeCursor".into()
                        },
                        serde_json::json!({ "value": cursor }),
                    )),
                    sorting_order
                        .map(|val| ("sortingOrder".into(), serde_json::json!({ "value": val }))),
                    sorting_field
                        .map(|val| ("sortingOrder".into(), serde_json::json!({ "value": val }))),
                    filter
                        .as_ref()
                        .map(|val| ("sortingOrder".into(), serde_json::json!({ "value": val }))),
                ]
                .into_iter()
                .flatten()
                .collect(),
            })
        }
        _ => Ok(()),
    }
}
