crate::data_type::ua_enum! {
    pub enum NodeClass {
        UNSPECIFIED = open62541_sys::UA_NodeClass::UA_NODECLASS_UNSPECIFIED,
        OBJECT = open62541_sys::UA_NodeClass::UA_NODECLASS_OBJECT,
        VARIABLE = open62541_sys::UA_NodeClass::UA_NODECLASS_VARIABLE,
        METHOD = open62541_sys::UA_NodeClass::UA_NODECLASS_METHOD,
        OBJECTTYPE = open62541_sys::UA_NodeClass::UA_NODECLASS_OBJECTTYPE,
        VARIABLETYPE = open62541_sys::UA_NodeClass::UA_NODECLASS_VARIABLETYPE,
        REFERENCETYPE = open62541_sys::UA_NodeClass::UA_NODECLASS_REFERENCETYPE,
        DATATYPE = open62541_sys::UA_NodeClass::UA_NODECLASS_DATATYPE,
        VIEW = open62541_sys::UA_NodeClass::UA_NODECLASS_VIEW,
    }
}
