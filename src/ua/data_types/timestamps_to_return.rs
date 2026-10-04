crate::data_type::ua_enum! {
    pub enum TimestampsToReturn {
        SOURCE = open62541_sys::UA_TimestampsToReturn::UA_TIMESTAMPSTORETURN_SOURCE,
        SERVER = open62541_sys::UA_TimestampsToReturn::UA_TIMESTAMPSTORETURN_SERVER,
        BOTH = open62541_sys::UA_TimestampsToReturn::UA_TIMESTAMPSTORETURN_BOTH,
        NEITHER = open62541_sys::UA_TimestampsToReturn::UA_TIMESTAMPSTORETURN_NEITHER,
        INVALID = open62541_sys::UA_TimestampsToReturn::UA_TIMESTAMPSTORETURN_INVALID,
    }
}
