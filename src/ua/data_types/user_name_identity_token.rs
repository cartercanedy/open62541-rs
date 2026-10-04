use crate::{DataType as _, ua};

crate::data_type::ua_struct! {
    pub struct UserNameIdentityToken {
        policyId: &ua::String,
        userName: &ua::String,
        password: &ua::ByteString,
        encryptionAlgorithm: &ua::String
    }
}

impl UserNameIdentityToken {
    /// Creates token with user name and password.
    ///
    /// This is a shortcut for calling [`Self::with_user_name()`] and [`Self::with_password()`].
    ///
    /// # Panics
    ///
    /// The user name must not contain any NUL bytes.
    #[must_use]
    pub fn new(user_name: &str, password: &str) -> Self {
        Self::init()
            .with_user_name(ua::String::new(user_name).unwrap())
            .with_password(ua::ByteString::new(password.as_bytes()))
    }
}
