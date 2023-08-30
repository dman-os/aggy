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

pub use cheapstr::CHeapStr;

mod cheapstr {
    use crate::interlude::*;

    use std::{
        borrow::Cow,
        hash::{Hash, Hasher},
    };
    // lifted from github.com/bevyengine/bevy 's bevy_core/Name struct
    // MIT/APACHE2 licence
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(crate = "serde", from = "String", into = "String")]
    pub struct CHeapStr {
        hash: u64,
        string: Cow<'static, str>,
    }

    impl std::fmt::Display for CHeapStr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.string.fmt(f)
        }
    }

    impl CHeapStr {
        /// Creates a new [`IdUnique`] from any string-like type.
        pub fn new(string: impl Into<Cow<'static, str>>) -> Self {
            let string = string.into();
            let mut id = Self { string, hash: 0 };
            id.update_hash();
            id
        }

        /// Gets the name of the entity as a `&str`.
        #[inline]
        pub fn as_str(&self) -> &str {
            &self.string
        }

        fn update_hash(&mut self) {
            let mut hasher = ahash::AHasher::default();
            self.string.hash(&mut hasher);
            self.hash = hasher.finish();
        }
    }

    impl From<&str> for CHeapStr {
        #[inline(always)]
        fn from(string: &str) -> Self {
            Self::new(string.to_owned())
        }
    }

    impl Hash for CHeapStr {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.string.hash(state);
        }
    }

    impl PartialEq for CHeapStr {
        fn eq(&self, other: &Self) -> bool {
            if self.hash != other.hash {
                // Makes the common case of two strings not been equal very fast
                return false;
            }

            self.string.eq(&other.string)
        }
    }

    impl Eq for CHeapStr {}

    impl PartialOrd for CHeapStr {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.string.partial_cmp(&other.string)
        }
    }

    impl Ord for CHeapStr {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.string.cmp(&other.string)
        }
    }

    impl std::ops::Deref for CHeapStr {
        type Target = Cow<'static, str>;

        fn deref(&self) -> &Self::Target {
            &self.string
        }
    }

    impl From<String> for CHeapStr {
        fn from(string: String) -> Self {
            /* let byte_arc: Arc<[u8]> = string.into_bytes().into();
            let str_arc = unsafe { Arc::from_raw(Arc::into_raw(byte_arc) as *const str) }; */
            Self::new(string)
        }
    }

    impl From<CHeapStr> for String {
        fn from(val: CHeapStr) -> String {
            // FIXME: optmize this
            /* let string = if let Some(s) = Arc::get_mut(&mut self.0) {
                unsafe {
                    String::from_raw_parts(
                        s as *mut str as *mut u8,
                        s.len(),
                        s.len()
                    )
                }
            } else {
                (&self.0[..]).to_string()
            };
            std::mem::forget(self.0);
            string */
            val.string.into_owned()
        }
    }
}
