crate::data_type::ua_optionset! {
    #[ua_inner = Byte]
    pub enum EventNotifierType {
        NONE = open62541_sys::UA_EVENTNOTIFIERTYPE_NONE,

        /// Indicates if it can be used to subscribe to events.
        SubscribeToEvents = open62541_sys::UA_EVENTNOTIFIERTYPE_SUBSCRIBETOEVENTS,
        /// Indicates if the history of the events is readable.
        HistoryRead = open62541_sys::UA_EVENTNOTIFIERTYPE_HISTORYREAD,
        /// Indicates if the history of the events is writeable.
        HistoryWrite = open62541_sys::UA_EVENTNOTIFIERTYPE_HISTORYWRITE,
    }
}
