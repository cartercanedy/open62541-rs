crate::data_type::ua_optionset! {
    #[ua_inner = UInt32]
    pub enum BrowseResultMask {
        NONE = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_NONE.0,
        ReferenceTypeId = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_REFERENCETYPEID.0,
        IsForward = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_ISFORWARD.0,
        NodeClass = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_NODECLASS.0,
        BrowseName = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_BROWSENAME.0,
        DisplayName = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_DISPLAYNAME.0,
        TypeDefinition = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_TYPEDEFINITION.0,
        All = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_ALL.0,
        ReferenceTypeInfo = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_REFERENCETYPEINFO.0,
        TargetInfo = open62541_sys::UA_BrowseResultMask::UA_BROWSERESULTMASK_TARGETINFO.0,
    }
}

#[cfg(test)]
mod tests {
    use crate::ua;

    #[test]
    fn combine_masks() {
        // We support mask combinators in `const` expressions.
        const LHS: ua::BrowseResultMask =
            ua::BrowseResultMask::REFERENCETYPEID.or(&ua::BrowseResultMask::ISFORWARD);
        const RHS: ua::BrowseResultMask = ua::BrowseResultMask::REFERENCETYPEINFO;
        assert_eq!(LHS, RHS);

        // We support mask combinators with `|` shorthand notation.
        let lhs = ua::BrowseResultMask::REFERENCETYPEID | ua::BrowseResultMask::ISFORWARD;
        let rhs = ua::BrowseResultMask::REFERENCETYPEINFO;
        assert_eq!(lhs, rhs);
    }
}
