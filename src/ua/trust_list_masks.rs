crate::data_type::ua_optionset! {
    #[ua_inner = UInt32]
    pub enum TrustListMasks {
        NONE = open62541_sys::UA_TrustListMasks::UA_TRUSTLISTMASKS_NONE.0,
        TrustedCertificates = open62541_sys::UA_TrustListMasks::UA_TRUSTLISTMASKS_TRUSTEDCERTIFICATES.0,
        TrustedCrls = open62541_sys::UA_TrustListMasks::UA_TRUSTLISTMASKS_TRUSTEDCRLS.0,
        IssuerCertificates = open62541_sys::UA_TrustListMasks::UA_TRUSTLISTMASKS_ISSUERCERTIFICATES.0,
        IssuerCrls = open62541_sys::UA_TrustListMasks::UA_TRUSTLISTMASKS_ISSUERCRLS.0,
        All = open62541_sys::UA_TrustListMasks::UA_TRUSTLISTMASKS_ALL.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::ua;

    #[test]
    fn combine_masks() {
        // We support mask combinators in `const` expressions.
        const LHS: ua::TrustListMasks = ua::TrustListMasks::TRUSTEDCERTIFICATES
            .or(&ua::TrustListMasks::TRUSTEDCRLS)
            .or(&ua::TrustListMasks::ISSUERCERTIFICATES)
            .or(&ua::TrustListMasks::ISSUERCRLS);
        const RHS: ua::TrustListMasks = ua::TrustListMasks::ALL;
        assert_eq!(LHS, RHS);

        // We support mask combinators with `|` shorthand notation.
        let lhs = ua::TrustListMasks::TRUSTEDCERTIFICATES
            | ua::TrustListMasks::TRUSTEDCRLS
            | ua::TrustListMasks::ISSUERCERTIFICATES
            | ua::TrustListMasks::ISSUERCRLS;
        let rhs = ua::TrustListMasks::ALL;
        assert_eq!(lhs, rhs);
    }
}
