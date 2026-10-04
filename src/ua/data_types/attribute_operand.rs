use crate::{FilterOperand, ua};

crate::data_type::ua_struct! {
    pub struct AttributeOperand {
        indexRange: &ua::String,
        nodeId: &ua::NodeId,
        browsePath: &ua::RelativePath,
        alias: &ua::String,
        #[from_inner]
        attributeId: ua::AttributeId
    }
}

impl FilterOperand for AttributeOperand {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        ua::ExtensionObject::new(self)
    }
}
