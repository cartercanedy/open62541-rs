use crate::{DataType as _, ServiceResponse, ua};

crate::data_type!(TranslateBrowsePathsToNodeIdsResponse);

impl TranslateBrowsePathsToNodeIdsResponse {
    pub fn node_ids(&self) -> impl ExactSizeIterator<Item = &ua::NodeId> {
        let results: &[ua::BrowsePathResult] = unsafe {
            ua::Array::slice_from_raw_parts(self.0.resultsSize, self.0.results).unwrap_or(&[])
        };

        
    }
}

impl ServiceResponse for TranslateBrowsePathsToNodeIdsResponse {
    type Request = ua::TranslateBrowsePathsToNodeIdsRequest;

    fn response_header(&self) -> &ua::ResponseHeader {
        ua::ResponseHeader::raw_ref(&self.0.responseHeader)
    }
}

