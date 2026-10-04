use crate::ua;

crate::data_type::ua_struct! {
    pub struct IssuedIdentityToken {
        policyId: &ua::String,
        tokenData: &ua::ByteString,
        encryptionAlgorithm: &ua::String
    }
}
