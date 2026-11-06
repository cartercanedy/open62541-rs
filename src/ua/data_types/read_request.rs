use crate::{DataType as _, ServiceRequest, ua};

crate::data_type::ua_struct! {
    pub struct ReadRequest {
        timestampsToReturn: &ua::TimestampsToReturn,
        nodesToRead: [ua::ReadValueId],
        requestHeader: &mut ua::RequestHeader
    }
}

impl ServiceRequest for ReadRequest {
    type Response = ua::ReadResponse;

    fn request_header(&self) -> &ua::RequestHeader {
        ua::RequestHeader::raw_ref(&self.0.requestHeader)
    }

    fn request_header_mut(&mut self) -> &mut ua::RequestHeader {
        // SAFETY: The request exclusively owns its initialized header.
        unsafe { ua::RequestHeader::raw_mut(&mut self.0.requestHeader) }
    }
}
