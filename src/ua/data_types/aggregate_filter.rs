use crate::{MonitoringFilter, ua};

crate::data_type::ua_struct! {
    pub struct AggregateFilter {
        aggregateType: &ua::NodeId,
        #[from_inner]
        startTime: ua::DateTime,
        #[builder(skip)]
        #[skip]
        aggregateConfiguration: open62541_sys::UA_AggregateConfiguration
    }
}

impl MonitoringFilter for AggregateFilter {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        ua::ExtensionObject::new(self)
    }
}
