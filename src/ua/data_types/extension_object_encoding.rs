use open62541_sys::UA_ExtensionObjectEncoding;

crate::data_type::ua_enum! {
    #[ua_inner = UInt32]
    pub enum ExtensionObjectEncoding {
        ENCODED_NOBODY = UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_ENCODED_NOBODY,
        ENCODED_BYTESTRING = UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_ENCODED_BYTESTRING,
        ENCODED_XML = UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_ENCODED_XML,
        DECODED = UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_DECODED,
        DECODED_NODELETE = UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_DECODED_NODELETE,
    }
}

impl ExtensionObjectEncoding {
    #[must_use]
    #[deprecated(note = "use unsafe `from_ua_unchecked` instead")]
    pub const fn from_ua(value: &UA_ExtensionObjectEncoding) -> Self {
        // SAFETY: This raw integer conversion has no additional preconditions.
        unsafe { Self::from_ua_unchecked(value) }
    }

    #[must_use]
    #[deprecated(note = "use unsafe `as_u32_unchecked` instead")]
    pub const fn as_u32(self) -> u32 {
        // SAFETY: Reading the raw integer has no additional preconditions.
        unsafe { self.as_u32_unchecked() }
    }
}
