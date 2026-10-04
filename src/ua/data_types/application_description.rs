use crate::ua;

crate::data_type::ua_struct! {
    pub struct ApplicationDescription {
        applicationUri: &ua::String,
        productUri: &ua::String,
        applicationName: &ua::LocalizedText,
        applicationType: &ua::ApplicationType,
        gatewayServerUri: &ua::String,
        discoveryProfileUri: &ua::String,
        discoveryUrls: [ua::String]
    }
}
