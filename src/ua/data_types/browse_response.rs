use crate::{DataType as _, ServiceResponse, ua};

crate::data_type::ua_struct! {
    pub struct BrowseResponse {
        results: [ua::BrowseResult],
        responseHeader: &ua::ResponseHeader,
        diagnosticInfos: [ua::DiagnosticInfo]
    }
}

impl ServiceResponse for BrowseResponse {
    type Request = ua::BrowseRequest;

    fn response_header(&self) -> &ua::ResponseHeader {
        ua::ResponseHeader::raw_ref(&self.0.responseHeader)
    }
}
