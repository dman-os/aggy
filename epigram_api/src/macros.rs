/// Implement [`From`] [`crate::auth::authorize::Error`] for the provided type
/// This expects the standard unit `AccessDenied` and the struct `Internal`
/// variant on the `Error` enum
#[macro_export]
macro_rules! impl_from_auth_err {
    ($errty:ident) => {
        impl From<$crate::auth::authorize::Error> for $errty {
            fn from(err: $crate::auth::authorize::Error) -> Self {
                use $crate::auth::authorize::Error;
                match err {
                    Error::Unauthorized | Error::InvalidToken => Self::AccessDenied,
                    Error::Internal { message } => Self::Internal { message },
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_from_service_auth_err {
    ($errty:ident) => {
        impl From<$crate::auth::authorize_service::Error> for $errty {
            fn from(err: $crate::auth::authorize_service::Error) -> Self {
                use $crate::auth::authorize_service::Error;
                match err {
                    Error::Unauthorized | Error::InvalidToken => Self::AccessDenied,
                    Error::Internal { message } => Self::Internal { message },
                }
            }
        }
    };
}
