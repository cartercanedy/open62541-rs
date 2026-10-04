use open62541_sys::{UA_MonitoredItemCreateRequest_default, UA_NODEID_NUMERIC};

use crate::ua;

crate::data_type::ua_struct! {
    pub struct MonitoredItemCreateRequest {
        itemToMonitor: &ua::ReadValueId,
        monitoringMode: &ua::MonitoringMode,
        requestedParameters: &mut ua::MonitoringParameters
    }
}

impl MonitoredItemCreateRequest {
    #[must_use]
    pub(crate) const fn attribute_id(&self) -> ua::AttributeId {
        self.item_to_monitor().attribute_id()
    }
}

impl Default for MonitoredItemCreateRequest {
    fn default() -> Self {
        // Use default node ID that does not own any additional data (such as dynamically allocated
        // string identifiers).
        let inner = unsafe { UA_MonitoredItemCreateRequest_default(UA_NODEID_NUMERIC(0, 0)) };
        Self(inner)
    }
}
