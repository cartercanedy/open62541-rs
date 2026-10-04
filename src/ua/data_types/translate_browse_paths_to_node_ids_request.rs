use crate::{ServiceRequest, ua};

crate::data_type::ua_struct! {
    pub struct TranslateBrowsePathsToNodeIdsRequest {
        browsePaths: [ua::BrowsePath],
        requestHeader: &mut ua::RequestHeader
    }
}

impl ServiceRequest for TranslateBrowsePathsToNodeIdsRequest {
    type Response = ua::TranslateBrowsePathsToNodeIdsResponse;

    fn request_header(&self) -> &ua::RequestHeader {
        Self::request_header(self)
    }

    fn request_header_mut(&mut self) -> &mut ua::RequestHeader {
        Self::request_header_mut(self)
    }
}
