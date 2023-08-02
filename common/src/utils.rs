use crate::interlude::*;

mod validation_errs;
pub use validation_errs::*;

pub mod testing;

mod list_request;
pub use list_request::*;

pub fn encode_hex_multibase<T: AsRef<[u8]>>(source: T) -> String {
    format!(
        "f{}",
        data_encoding::HEXLOWER_PERMISSIVE.encode(source.as_ref())
    )
}

pub fn decode_hex_multibase(source: &str) -> eyre::Result<Vec<u8>> {
    match (
        &source[0..1],
        data_encoding::HEXLOWER_PERMISSIVE.decode(source[1..].as_bytes()),
    ) {
        ("f", Ok(bytes)) => Ok(bytes),
        (prefix, Ok(_)) => Err(eyre::eyre!(
            "unexpected multibase prefix for hex multibase: {prefix}"
        )),
        (_, Err(err)) => Err(eyre::eyre!("error decoding hex: {err}")),
    }
}

/// This baby doesn't work on generic types
pub fn type_name_raw<T>() -> &'static str {
    let name = std::any::type_name::<T>();
    match &name.rfind(':') {
        Some(pos) => &name[pos + 1..name.len()],
        None => name,
    }
}

#[test]
fn test_type_name_macro() {
    struct Foo {}
    assert_eq!("Foo", type_name_raw::<Foo>());
}

/*
/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    use serde::Deserialize;
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => std::str::FromStr::from_str(s).map_err(serde::de::Error::custom).map(Some),
    }
}
*/
pub fn get_env_var<K>(key: K) -> eyre::Result<String>
where
    K: AsRef<std::ffi::OsStr>,
{
    match std::env::var(key.as_ref()) {
        Ok(val) => Ok(val),
        Err(err) => Err(eyre::eyre!(
            "error geting env var {:?}: {err}",
            key.as_ref()
        )),
    }
}

#[inline]
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
