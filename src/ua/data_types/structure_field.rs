use crate::ua;

crate::data_type::ua_struct! {
    pub struct StructureField {
        name: &ua::String,
        description: &ua::LocalizedText,
        dataType: &ua::NodeId,
        valueRank: i32,
        arrayDimensions: [ua::UInt32],
        maxStringLength: u32,
        isOptional: bool
    }
}
