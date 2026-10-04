use crate::ua;

crate::data_type::ua_struct! {
    pub struct RelativePathElement {
        referenceTypeId: &ua::NodeId,
        isInverse: bool,
        includeSubtypes: bool,
        targetName: &ua::QualifiedName
    }
}
