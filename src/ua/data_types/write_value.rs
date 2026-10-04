use crate::ua;

crate::data_type::ua_struct! {
    pub struct WriteValue {
        nodeId: &ua::NodeId,
        #[from_inner]
        attributeId: ua::AttributeId,
        value: &ua::DataValue,
        indexRange: &ua::String
    }
}
