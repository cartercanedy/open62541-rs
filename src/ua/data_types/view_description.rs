use crate::ua;

crate::data_type::ua_struct! {
    pub struct ViewDescription {
        viewId: &ua::NodeId,
        viewVersion: u32,
        #[from_inner]
        timestamp: ua::DateTime
    }
}
