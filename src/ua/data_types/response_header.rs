use crate::ua;

crate::data_type::ua_struct! {
    pub struct ResponseHeader {
        additionalHeader: &ua::ExtensionObject,
        requestHandle: u32,
        #[from_inner]
        timestamp: ua::DateTime,
        serviceDiagnostics: &ua::DiagnosticInfo,
        #[from_inner]
        serviceResult: ua::StatusCode,
        stringTable: [ua::String]
    }
}
