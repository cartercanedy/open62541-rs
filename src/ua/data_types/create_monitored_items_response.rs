use crate::ua;

crate::data_type::ua_struct! {
    pub struct CreateMonitoredItemsResponse {
        results: [ua::MonitoredItemCreateResult],
        diagnosticInfos: [ua::DiagnosticInfo],
        responseHeader: &ua::ResponseHeader
    }
}
