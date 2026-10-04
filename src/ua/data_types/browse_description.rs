use open62541_sys::UA_NS0ID_HIERARCHICALREFERENCES;

use crate::{DataType, ua};

crate::data_type::ua_struct! {
    pub struct BrowseDescription {
        nodeId: &ua::NodeId,
        referenceTypeId: &ua::NodeId,
        includeSubtypes: bool,
        #[from_inner]
        nodeClassMask: ua::NodeClassMask,
        #[from_inner]
        resultMask: ua::BrowseResultMask,
        browseDirection: &ua::BrowseDirection
    }
}

impl Default for BrowseDescription {
    fn default() -> Self {
        Self::init()
            .with_browse_direction(ua::BrowseDirection::FORWARD)
            .with_reference_type_id(ua::NodeId::numeric(0, UA_NS0ID_HIERARCHICALREFERENCES))
            .with_include_subtypes(true)
            .with_result_mask(ua::BrowseResultMask::ALL)
    }
}
