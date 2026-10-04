use crate::ua;

crate::data_type::ua_struct! {
    pub struct BrowsePathResult {
        targets: [ua::BrowsePathTarget],
        #[from_inner]
        statusCode: ua::StatusCode
    }
}
