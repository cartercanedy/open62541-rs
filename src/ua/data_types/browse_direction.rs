crate::data_type::ua_enum! {
    pub enum BrowseDirection {
        FORWARD = open62541_sys::UA_BrowseDirection::UA_BROWSEDIRECTION_FORWARD,
        INVERSE = open62541_sys::UA_BrowseDirection::UA_BROWSEDIRECTION_INVERSE,
        BOTH = open62541_sys::UA_BrowseDirection::UA_BROWSEDIRECTION_BOTH,
        INVALID = open62541_sys::UA_BrowseDirection::UA_BROWSEDIRECTION_INVALID,
    }
}
