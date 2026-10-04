use crate::ua;

crate::data_type::ua_struct! {
    pub struct CallMethodResult {
        #[from_inner]
        statusCode: ua::StatusCode,
        inputArgumentResults: [ua::StatusCode],
        outputArguments: [ua::Variant]
    }
}
