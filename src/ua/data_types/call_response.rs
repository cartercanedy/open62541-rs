use crate::{ServiceResponse, ua};

crate::data_type::ua_struct! {
    pub struct CallResponse {
        results: [ua::CallMethodResult],
        responseHeader: &ua::ResponseHeader
    }
}

impl ServiceResponse for CallResponse {
    type Request = ua::CallRequest;

    fn response_header(&self) -> &ua::ResponseHeader {
        Self::response_header(self)
    }
}
