use crate::{FilterOperand, ua};

crate::data_type::ua_struct! {
    pub struct ElementOperand { index: u32 }
}

impl FilterOperand for ElementOperand {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        ua::ExtensionObject::new(self)
    }
}
