use crate::{DataType as _, ServiceRequest, ua};

crate::data_type::ua_struct! {
    pub struct BrowseNextRequest {
        continuationPoints: [ua::ContinuationPoint],
        requestHeader: &mut ua::RequestHeader,
        releaseContinuationPoints: bool
    }
}

impl ServiceRequest for BrowseNextRequest {
    type Response = ua::BrowseNextResponse;

    fn request_header(&self) -> &ua::RequestHeader {
        ua::RequestHeader::raw_ref(&self.0.requestHeader)
    }

    fn request_header_mut(&mut self) -> &mut ua::RequestHeader {
        ua::RequestHeader::raw_mut(&mut self.0.requestHeader)
    }
}
