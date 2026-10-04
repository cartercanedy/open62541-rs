use open62541_sys::UA_StructureDescription;

use crate::{DataType as _, ua};

crate::data_type::ua_struct! {
    pub struct StructureDescription {
        dataTypeId: &ua::NodeId,
        name: &ua::QualifiedName,
        structureDefinition: &ua::StructureDefinition
    }
}

impl StructureDescription {
    pub(crate) fn new(
        data_type_id: ua::NodeId,
        name: ua::QualifiedName,
        definition: ua::StructureDefinition,
    ) -> Self {
        Self(UA_StructureDescription {
            dataTypeId: data_type_id.into_raw(),
            name: name.into_raw(),
            structureDefinition: definition.into_raw(),
        })
    }
}
