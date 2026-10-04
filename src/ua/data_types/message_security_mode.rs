crate::data_type::ua_enum! {
    pub enum MessageSecurityMode {
        INVALID = open62541_sys::UA_MessageSecurityMode::UA_MESSAGESECURITYMODE_INVALID,
        NONE = open62541_sys::UA_MessageSecurityMode::UA_MESSAGESECURITYMODE_NONE,
        SIGN = open62541_sys::UA_MessageSecurityMode::UA_MESSAGESECURITYMODE_SIGN,
        SIGNANDENCRYPT = open62541_sys::UA_MessageSecurityMode::UA_MESSAGESECURITYMODE_SIGNANDENCRYPT,
    }
}
