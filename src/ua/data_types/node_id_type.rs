crate::data_type::ua_enum! {
    #[ua_inner = UInt32]
    pub enum NodeIdType {
        NUMERIC = open62541_sys::UA_NodeIdType::UA_NODEIDTYPE_NUMERIC,
        STRING = open62541_sys::UA_NodeIdType::UA_NODEIDTYPE_STRING,
        GUID = open62541_sys::UA_NodeIdType::UA_NODEIDTYPE_GUID,
        BYTESTRING = open62541_sys::UA_NodeIdType::UA_NODEIDTYPE_BYTESTRING,
    }
}
