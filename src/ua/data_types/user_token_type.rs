crate::data_type::ua_enum! {
    pub enum UserTokenType {
        ANONYMOUS = open62541_sys::UA_UserTokenType::UA_USERTOKENTYPE_ANONYMOUS,
        USERNAME = open62541_sys::UA_UserTokenType::UA_USERTOKENTYPE_USERNAME,
        CERTIFICATE = open62541_sys::UA_UserTokenType::UA_USERTOKENTYPE_CERTIFICATE,
        ISSUEDTOKEN = open62541_sys::UA_UserTokenType::UA_USERTOKENTYPE_ISSUEDTOKEN,
    }
}
