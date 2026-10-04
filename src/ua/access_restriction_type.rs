// See <https://reference.opcfoundation.org/Core/Part3/v105/docs/8.56> for bit values.
crate::data_type::ua_optionset! {
    #[ua_inner = UInt16]
    pub enum AccessRestrictionType {
        NONE = open62541_sys::UA_ACCESSRESTRICTIONTYPE_NONE,

        /// If this bit is set, the access restrictions `SigningRequired` and `EncryptionRequired` are
        /// also applied to the browse permission.
        ApplyRestrictionsToBrowse = open62541_sys::UA_ACCESSRESTRICTIONTYPE_APPLYRESTRICTIONSTOBROWSE,
        /// The client can only access the node when using a secure channel which encrypts all messages.
        ///
        /// This does not apply to the browse permission if the `ApplyRestrictionsToBrowse` is not set.
        EncryptionRequired = open62541_sys::UA_ACCESSRESTRICTIONTYPE_ENCRYPTIONREQUIRED,
        /// The client cannot access the node when using `SessionlessInvoke` service invocation.
        SessionRequired = open62541_sys::UA_ACCESSRESTRICTIONTYPE_SESSIONREQUIRED,
        /// The client can only access the node when using a secure channel which digitally signs all
        /// messages.
        ///
        /// This does not apply to the browse permission if the `ApplyRestrictionsToBrowse` is not set.
        SigningRequired = open62541_sys::UA_ACCESSRESTRICTIONTYPE_SIGNINGREQUIRED,
    }
}
