crate::data_type::ua_enum! {
    pub enum MonitoringMode {
        DISABLED = open62541_sys::UA_MonitoringMode::UA_MONITORINGMODE_DISABLED,
        SAMPLING = open62541_sys::UA_MonitoringMode::UA_MONITORINGMODE_SAMPLING,
        REPORTING = open62541_sys::UA_MonitoringMode::UA_MONITORINGMODE_REPORTING,
    }
}
