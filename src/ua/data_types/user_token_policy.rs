use crate::ua;

crate::data_type::ua_struct! {
    pub struct UserTokenPolicy {
        policyId: &ua::String,
        tokenType: &ua::UserTokenType,
        issuedTokenType: &ua::String,
        issuerEndpointUrl: &ua::String,
        securityPolicyUri: &ua::String
    }
}
