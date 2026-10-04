use crate::ua;

crate::data_type::ua_struct! {
    pub struct EnumField {
        description: &ua::LocalizedText,
        displayName: &ua::LocalizedText,
        name: &ua::String,
        value: i64
    }
}
