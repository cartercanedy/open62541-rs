use crate::ua;

crate::data_type::ua_struct! {
    pub struct EndpointDescription {
        endpointUrl: &ua::String,
        securityPolicyUri: &ua::String,
        transportProfileUri: &ua::String,
        server: &ua::ApplicationDescription,
        serverCertificate: &ua::ByteString,
        securityMode: &ua::MessageSecurityMode,
        #[from_inner]
        securityLevel: ua::SecurityLevel,
        userIdentityTokens: [ua::UserTokenPolicy]
    }
}
