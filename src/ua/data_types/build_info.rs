use crate::ua;

crate::data_type::ua_struct! {
    pub struct BuildInfo {
        #[from_inner]
        buildDate: ua::DateTime,
        buildNumber: &ua::String,
        manufacturerName: &ua::String,
        productName: &ua::String,
        productUri: &ua::String,
        softwareVersion: &ua::String
    }
}
