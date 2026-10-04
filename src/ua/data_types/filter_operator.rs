use std::hash;

crate::data_type::ua_enum! {
    pub enum FilterOperator {
        EQUALS = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_EQUALS,
        ISNULL = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_ISNULL,
        GREATERTHAN = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_GREATERTHAN,
        LESSTHAN = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_LESSTHAN,
        GREATERTHANOREQUAL = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_GREATERTHANOREQUAL,
        LESSTHANOREQUAL = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_LESSTHANOREQUAL,
        LIKE = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_LIKE,
        NOT = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_NOT,
        BETWEEN = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_BETWEEN,
        INLIST = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_INLIST,
        AND = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_AND,
        OR = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_OR,
        CAST = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_CAST,
        INVIEW = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_INVIEW,
        OFTYPE = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_OFTYPE,
        RELATEDTO = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_RELATEDTO,
        BITWISEAND = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_BITWISEAND,
        BITWISEOR = open62541_sys::UA_FilterOperator::UA_FILTEROPERATOR_BITWISEOR,
    }
}

impl hash::Hash for FilterOperator {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
