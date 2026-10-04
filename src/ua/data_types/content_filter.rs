use crate::ua;

crate::data_type::ua_struct! {
    pub struct ContentFilter {
        elements: [ua::ContentFilterElement]
    }
}
