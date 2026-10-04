use crate::{DataType as _, Error, ua};

crate::data_type::ua_struct! {
    pub struct LocalizedText {
        locale: &ua::String,
        text: &ua::String
    }
}

impl LocalizedText {
    /// Creates localized text from locale and text.
    ///
    /// # Errors
    ///
    /// The strings must not contain any NUL bytes.
    pub fn new(locale: &str, text: &str) -> Result<Self, Error> {
        Ok(Self::init()
            .with_locale(ua::String::new(locale)?)
            .with_text(ua::String::new(text)?))
    }
}
