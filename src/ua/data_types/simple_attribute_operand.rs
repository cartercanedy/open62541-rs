use crate::{FilterOperand, ua};

crate::data_type::ua_struct! {
    pub struct SimpleAttributeOperand {
        typeDefinitionId: &ua::NodeId,
        browsePath: [ua::QualifiedName],
        #[from_inner]
        attributeId: ua::AttributeId,
        indexRange: &ua::String
    }
}

impl FilterOperand for SimpleAttributeOperand {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        ua::ExtensionObject::new(self)
    }
}
