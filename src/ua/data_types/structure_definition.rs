use crate::ua;

crate::data_type::ua_struct! {
    pub struct StructureDefinition {
        defaultEncodingId: &ua::NodeId,
        baseDataType: &ua::NodeId,
        structureType: &ua::StructureType,
        fields: [ua::StructureField]
    }
}

impl StructureDefinition {
    #[must_use]
    pub fn into_description(
        self,
        data_type_id: ua::NodeId,
        name: ua::QualifiedName,
    ) -> ua::StructureDescription {
        ua::StructureDescription::new(data_type_id, name, self)
    }
}
