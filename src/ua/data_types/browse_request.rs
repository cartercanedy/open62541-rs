use crate::{DataType, ServiceRequest, ua};

crate::data_type::ua_struct! {
    pub struct BrowseRequest {
        nodesToBrowse: [ua::BrowseDescription],
        requestedMaxReferencesPerNode: u32,
        view: &ua::ViewDescription,
        requestHeader: &mut ua::RequestHeader
    }
}

impl ServiceRequest for BrowseRequest {
    type Response = ua::BrowseResponse;

    fn request_header(&self) -> &ua::RequestHeader {
        ua::RequestHeader::raw_ref(&self.0.requestHeader)
    }

    fn request_header_mut(&mut self) -> &mut ua::RequestHeader {
        ua::RequestHeader::raw_mut(&mut self.0.requestHeader)
    }
}
