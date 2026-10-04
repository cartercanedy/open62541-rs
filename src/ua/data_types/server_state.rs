crate::data_type::ua_enum! {
    pub enum ServerState {
        FAILED = open62541_sys::UA_ServerState::UA_SERVERSTATE_FAILED,
        NOCONFIGURATION = open62541_sys::UA_ServerState::UA_SERVERSTATE_NOCONFIGURATION,
        SUSPENDED = open62541_sys::UA_ServerState::UA_SERVERSTATE_SUSPENDED,
        RUNNING = open62541_sys::UA_ServerState::UA_SERVERSTATE_RUNNING,
        SHUTDOWN = open62541_sys::UA_ServerState::UA_SERVERSTATE_SHUTDOWN,
        TEST = open62541_sys::UA_ServerState::UA_SERVERSTATE_TEST,
        COMMUNICATIONFAULT = open62541_sys::UA_ServerState::UA_SERVERSTATE_COMMUNICATIONFAULT,
        UNKNOWN = open62541_sys::UA_ServerState::UA_SERVERSTATE_UNKNOWN,
    }
}
