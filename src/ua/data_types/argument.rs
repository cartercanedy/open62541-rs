use crate::{ValueType, ua};

crate::data_type::ua_struct! {
    pub struct Argument {
        dataType: &ua::NodeId,
        name: &ua::String,
        description: &ua::LocalizedText,
        valueRank: i32,
        arrayDimensions: [ua::UInt32]
    }
}

impl Argument {
    /// Classifies the declared datatype by its `NodeId` without resolving subtype ancestry.
    ///
    /// Unlike [`ua::Variant::value_type()`], this has no native descriptor to recognize enums.
    #[must_use]
    pub fn value_type(&self) -> ValueType {
        ValueType::from_data_type(self.data_type())
    }
}
