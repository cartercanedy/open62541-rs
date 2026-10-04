// See <https://reference.opcfoundation.org/Core/Part3/v104/docs/8.57> for bit values.
crate::data_type::ua_optionset! {
    #[ua_inner = Byte]
    pub enum AccessLevelType {
        NONE = open62541_sys::UA_ACCESSLEVELTYPE_NONE,

        /// Indicates if the current value is readable.
        ///
        /// It also indicates if the current value of the variable is available.
        CurrentRead = open62541_sys::UA_ACCESSLEVELTYPE_CURRENTREAD,
        /// Indicates if the current value is writable.
        ///
        /// It also indicates if the current value of the variable is available.
        CurrentWrite = open62541_sys::UA_ACCESSLEVELTYPE_CURRENTWRITE,
        /// Indicates if the history of the value is readable.
        ///
        /// It also indicates if the history of the variable is available via the OPC UA Server.
        HistoryRead = open62541_sys::UA_ACCESSLEVELTYPE_HISTORYREAD,
        /// Indicates if the history of the value is writable.
        ///
        /// It also indicates if the history of the variable is available via the OPC UA Server.
        HistoryWrite = open62541_sys::UA_ACCESSLEVELTYPE_HISTORYWRITE,
        /// This flag is set for properties that define semantic aspects of the parent node of the
        /// property and where the property value, and thus the semantic, may change during operation.
        SemanticChange = open62541_sys::UA_ACCESSLEVELTYPE_SEMANTICCHANGE,
        /// Indicates if the current status code of the value is writable.
        StatusWrite = open62541_sys::UA_ACCESSLEVELTYPE_STATUSWRITE,
        /// Indicates if the current source timestamp is writable.
        TimestampWrite = open62541_sys::UA_ACCESSLEVELTYPE_TIMESTAMPWRITE,
    }
}

#[deprecated = "Use AccessLevelType instead."]
pub type AccessLevel = AccessLevelType;
