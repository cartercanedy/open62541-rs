crate::data_type::ua_enum! {
    pub enum DataChangeTrigger {
        STATUS = open62541_sys::UA_DataChangeTrigger::UA_DATACHANGETRIGGER_STATUS,
        STATUSVALUE = open62541_sys::UA_DataChangeTrigger::UA_DATACHANGETRIGGER_STATUSVALUE,
        STATUSVALUETIMESTAMP = open62541_sys::UA_DataChangeTrigger::UA_DATACHANGETRIGGER_STATUSVALUETIMESTAMP,
    }
}
