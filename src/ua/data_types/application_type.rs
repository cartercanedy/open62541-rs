crate::data_type::ua_enum! {
    pub enum ApplicationType {
        SERVER = open62541_sys::UA_ApplicationType::UA_APPLICATIONTYPE_SERVER,
        CLIENT = open62541_sys::UA_ApplicationType::UA_APPLICATIONTYPE_CLIENT,
        CLIENTANDSERVER = open62541_sys::UA_ApplicationType::UA_APPLICATIONTYPE_CLIENTANDSERVER,
        DISCOVERYSERVER = open62541_sys::UA_ApplicationType::UA_APPLICATIONTYPE_DISCOVERYSERVER,
    }
}
