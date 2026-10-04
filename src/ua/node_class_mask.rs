use crate::ua;

crate::data_type::ua_optionset! {
    #[ua_inner = UInt32]
    pub enum NodeClassMask {
        NONE = 0u32,
        Object = ua::NodeClass::OBJECT_U32,
        Variable = ua::NodeClass::VARIABLE_U32,
        Method = ua::NodeClass::METHOD_U32,
        ObjectType = ua::NodeClass::OBJECTTYPE_U32,
        VariableType = ua::NodeClass::VARIABLETYPE_U32,
        ReferenceType = ua::NodeClass::REFERENCETYPE_U32,
        DataType = ua::NodeClass::DATATYPE_U32,
        View = ua::NodeClass::VIEW_U32,
    }
}
