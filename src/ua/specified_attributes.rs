use open62541_sys::UA_NodeAttributesMask;

crate::data_type::ua_optionset! {
    #[ua_inner = UInt32]
    #[ua_type = NodeAttributesMask]
    pub enum SpecifiedAttributes {
        NONE = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_NONE.0,
        /// Indicates if the `AccessLevel` Attribute is set.
        AccessLevel = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ACCESSLEVEL.0,
        /// Indicates if the `ArrayDimensions` Attribute is set.
        ArrayDimensions = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ARRAYDIMENSIONS.0,
        /// Indicates if the `ContainsNoLoops` Attribute is set.
        ContainsNoLoops = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_CONTAINSNOLOOPS.0,
        /// Indicates if the `DataType` Attribute is set.
        DataType = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DATATYPE.0,
        /// Indicates if the `Description` Attribute is set.
        Description = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DESCRIPTION.0,
        /// Indicates if the `DisplayName` Attribute is set.
        DisplayName = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DISPLAYNAME.0,
        /// Indicates if the `EventNotifier` Attribute is set.
        EventNotifier = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_EVENTNOTIFIER.0,
        /// Indicates if the `Executable` Attribute is set.
        Executable = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_EXECUTABLE.0,
        /// Indicates if the `Historizing` Attribute is set.
        Historizing = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_HISTORIZING.0,
        /// Indicates if the `InverseName` Attribute is set.
        InverseName = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_INVERSENAME.0,
        /// Indicates if the `IsAbstract` Attribute is set.
        IsAbstract = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ISABSTRACT.0,
        /// Indicates if the `MinimumSamplingInterval` Attribute is set.
        MinimumSamplingInterval = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_MINIMUMSAMPLINGINTERVAL.0,
        /// Indicates if the `Symmetric` Attribute is set.
        Symmetric = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_SYMMETRIC.0,
        /// Indicates if the `UserAccessLevel` Attribute is set.
        UserAccessLevel = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_USERACCESSLEVEL.0,
        /// Indicates if the `UserExecutable` Attribute is set.
        UserExecutable = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_USEREXECUTABLE.0,
        /// Indicates if the `UserWriteMask` Attribute is set.
        UserWriteMask = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_USERWRITEMASK.0,
        /// Indicates if the `ValueRank` Attribute is set.
        ValueRank = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_VALUERANK.0,
        /// Indicates if the `WriteMask` Attribute is set.
        WriteMask = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_WRITEMASK.0,
        /// Indicates if the `Value` Attribute is set.
        Value = UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_VALUE.0,
    }
}
