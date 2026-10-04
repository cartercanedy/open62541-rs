use crate::ua;

crate::data_type::ua_struct! {
    pub struct ContentFilterElement {
        filterOperator: &ua::FilterOperator,
        filterOperands: [ua::ExtensionObject]
    }
}
