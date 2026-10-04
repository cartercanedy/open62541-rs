use crate::ua;

crate::data_type::ua_struct! {
    pub struct ReadValueId {
        nodeId: &ua::NodeId,
        #[from_inner]
        attributeId: ua::AttributeId,
        dataEncoding: &ua::QualifiedName,
        indexRange: &ua::String
    }
}
