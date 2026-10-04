use crate::{MonitoringFilter, ua};

crate::data_type::ua_struct! {
    pub struct EventFilter {
        selectClauses: [ua::SimpleAttributeOperand],
        whereClause: &ua::ContentFilter
    }
}

impl MonitoringFilter for EventFilter {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        ua::ExtensionObject::new(self)
    }
}
