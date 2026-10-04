// See <https://reference.opcfoundation.org/Core/Part3/v105/docs/8.60> for bit values.
crate::data_type::ua_optionset! {
    #[ua_inner = UInt32]
    pub enum AttributeWriteMask {
        NONE = open62541_sys::UA_ATTRIBUTEWRITEMASK_NONE,

        /// Indicates if the `AccessLevel` attribute is writeable.
        AccessLevel = open62541_sys::UA_ATTRIBUTEWRITEMASK_ACCESSLEVEL,
        /// Indicates if the `AccessLevelEx` attribute is writeable.
        AccessLevelEx = open62541_sys::UA_ATTRIBUTEWRITEMASK_ACCESSLEVELEX,
        /// Indicates if the `AccessRestrictions` attribute is writeable.
        AccessRestrictions = open62541_sys::UA_ATTRIBUTEWRITEMASK_ACCESSRESTRICTIONS,
        /// Indicates if the `ArrayDimensions` attribute is writeable.
        ArrayDimensions = open62541_sys::UA_ATTRIBUTEWRITEMASK_ARRAYDIMENSIONS,
        /// Indicates if the `BrowseName` attribute is writeable.
        BrowseName = open62541_sys::UA_ATTRIBUTEWRITEMASK_BROWSENAME,
        /// Indicates if the `ContainsNoLoops` attribute is writeable.
        ContainsNoLoops = open62541_sys::UA_ATTRIBUTEWRITEMASK_CONTAINSNOLOOPS,
        /// Indicates if the `DataType` attribute is writeable.
        DataType = open62541_sys::UA_ATTRIBUTEWRITEMASK_DATATYPE,
        /// Indicates if the `DataTypeDefinition` attribute is writeable.
        DataTypeDefinition = open62541_sys::UA_ATTRIBUTEWRITEMASK_DATATYPEDEFINITION,
        /// Indicates if the `Description` attribute is writeable.
        Description = open62541_sys::UA_ATTRIBUTEWRITEMASK_DESCRIPTION,
        /// Indicates if the `DisplayName` attribute is writeable.
        DisplayName = open62541_sys::UA_ATTRIBUTEWRITEMASK_DISPLAYNAME,
        /// Indicates if the `EventNotifier` attribute is writeable.
        EventNotifier = open62541_sys::UA_ATTRIBUTEWRITEMASK_EVENTNOTIFIER,
        /// Indicates if the `Executable` attribute is writeable.
        Executable = open62541_sys::UA_ATTRIBUTEWRITEMASK_EXECUTABLE,
        /// Indicates if the `Historizing` attribute is writeable.
        Historizing = open62541_sys::UA_ATTRIBUTEWRITEMASK_HISTORIZING,
        /// Indicates if the `InverseName` attribute is writeable.
        InverseName = open62541_sys::UA_ATTRIBUTEWRITEMASK_INVERSENAME,
        /// Indicates if the `IsAbstract` attribute is writeable.
        IsAbstract = open62541_sys::UA_ATTRIBUTEWRITEMASK_ISABSTRACT,
        /// Indicates if the `MinimumSamplingInterval` attribute is writeable.
        MinimumSamplingInterval = open62541_sys::UA_ATTRIBUTEWRITEMASK_MINIMUMSAMPLINGINTERVAL,
        /// Indicates if the `NodeClass` attribute is writeable.
        NodeClass = open62541_sys::UA_ATTRIBUTEWRITEMASK_NODECLASS,
        /// Indicates if the `NodeId` attribute is writeable.
        NodeId = open62541_sys::UA_ATTRIBUTEWRITEMASK_NODEID,
        /// Indicates if the `RolePermissions` attribute is writeable.
        RolePermissions = open62541_sys::UA_ATTRIBUTEWRITEMASK_ROLEPERMISSIONS,
        /// Indicates if the `Symmetric` attribute is writeable.
        Symmetric = open62541_sys::UA_ATTRIBUTEWRITEMASK_SYMMETRIC,
        /// Indicates if the `UserAccessLevel` attribute is writeable.
        UserAccessLevel = open62541_sys::UA_ATTRIBUTEWRITEMASK_USERACCESSLEVEL,
        /// Indicates if the `UserExecutable` attribute is writeable.
        UserExecutable = open62541_sys::UA_ATTRIBUTEWRITEMASK_USEREXECUTABLE,
        /// Indicates if the `UserWriteMask` attribute is writeable.
        UserWriteMask = open62541_sys::UA_ATTRIBUTEWRITEMASK_USERWRITEMASK,
        /// Indicates if the `ValueForVariableType` attribute is writeable.
        ///
        /// Note: It does not apply for variables since this is handled by the `AccessLevel` and
        /// `UserAccessLevel` attributes for the Variable. For variables this bit shall be set to 0.
       ValueForVariableType = open62541_sys::UA_ATTRIBUTEWRITEMASK_VALUEFORVARIABLETYPE,
        /// Indicates if the `ValueRank` attribute is writeable.
        ValueRank = open62541_sys::UA_ATTRIBUTEWRITEMASK_VALUERANK,
        /// Indicates if the `WriteMask` attribute is writeable.
        WriteMask = open62541_sys::UA_ATTRIBUTEWRITEMASK_WRITEMASK,
    }
}
