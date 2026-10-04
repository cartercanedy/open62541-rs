use crate::ua;

crate::data_type::ua_struct! {
    pub struct CallMethodRequest {
        objectId: &ua::NodeId,
        methodId: &ua::NodeId,
        inputArguments: [ua::Variant]
    }
}
