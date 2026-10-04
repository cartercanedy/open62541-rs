use crate::ua;

crate::data_type::ua_struct! {
    pub struct ReferenceDescription {
        referenceTypeId: &ua::NodeId,
        isForward: bool,
        nodeId: &ua::ExpandedNodeId,
        browseName: &ua::QualifiedName,
        displayName: &ua::LocalizedText,
        typeDefinition: &ua::ExpandedNodeId,
        nodeClass: &ua::NodeClass
    }
}
