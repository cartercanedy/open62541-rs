use crate::{MonitoringFilter, ua};

crate::data_type::ua_struct! {
    pub struct DataChangeFilter {
        deadbandType: u32,
        deadbandValue: f64,
        trigger: &ua::DataChangeTrigger
    }
}

impl MonitoringFilter for DataChangeFilter {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        ua::ExtensionObject::new(self)
    }
}
