// See <https://reference.opcfoundation.org/Core/Part3/v105/docs/8.58> for bit values.
crate::data_type::ua_optionset! {
    #[ua_inner = UInt32]
    pub enum AccessLevelExType {
        NONE = open62541_sys::UA_ACCESSLEVELEXTYPE_NONE,

        /// Indicates if the value of the variable can be considered constant.
        Constant = open62541_sys::UA_ACCESSLEVELEXTYPE_CONSTANT,
        /// Indicates if the current value is readable.
        ///
        /// It also indicates if the current value of the variable is available.
        CurrentRead = open62541_sys::UA_ACCESSLEVELEXTYPE_CURRENTREAD,
        /// Indicates if the current value is writable.
        ///
        /// It also indicates if the current value of the variable is available.
        CurrentWrite = open62541_sys::UA_ACCESSLEVELEXTYPE_CURRENTWRITE,
        /// Indicates if the history of the value is readable.
        ///
        /// It also indicates if the history of the variable is available via the OPC UA Server.
        HistoryRead = open62541_sys::UA_ACCESSLEVELEXTYPE_HISTORYREAD,
        /// Indicates if the history of the value is writable.
        ///
        /// It also indicates if the history of the variable is available via the OPC UA Server.
        HistoryWrite = open62541_sys::UA_ACCESSLEVELEXTYPE_HISTORYWRITE,
        /// This flag is set for properties that define semantic aspects of the parent node of the
        /// property and where the property value, and thus the semantic, may change during operation.
        SemanticChange = open62541_sys::UA_ACCESSLEVELEXTYPE_SEMANTICCHANGE,
        /// Indicates if the current status code of the value is writable.
        StatusWrite = open62541_sys::UA_ACCESSLEVELEXTYPE_STATUSWRITE,
        /// Indicates if the current source timestamp is writable.
        TimestampWrite = open62541_sys::UA_ACCESSLEVELEXTYPE_TIMESTAMPWRITE,
        /// Indicates non-atomicity for read access.
        NonatomicRead = open62541_sys::UA_ACCESSLEVELEXTYPE_NONATOMICREAD,
        /// Indicates non-atomicity for write access.
        NonatomicWrite = open62541_sys::UA_ACCESSLEVELEXTYPE_NONATOMICWRITE,
        /// Indicates if write of index range is supported.
        WriteFullArrayOnly = open62541_sys::UA_ACCESSLEVELEXTYPE_WRITEFULLARRAYONLY,
        /// Indicates if the variable doesn’t allow its data type to be subtyped.
        NoSubDataTypes = open62541_sys::UA_ACCESSLEVELEXTYPE_NOSUBDATATYPES,
        /// Indicates if the variable is non-volatile.
        NonVolatile = open62541_sys::UA_ACCESSLEVELEXTYPE_NONVOLATILE,
    }
}
