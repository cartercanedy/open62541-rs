use crate::ua;

crate::data_type::ua_struct! {
    pub struct BrowsePath {
        startingNode: &ua::NodeId,
        relativePath: &ua::RelativePath
    }
}
