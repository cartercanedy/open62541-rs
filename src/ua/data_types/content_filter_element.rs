use open62541_sys::UA_FilterOperator;

use crate::{DataType, FilterOperand, ua};

crate::data_type!(ContentFilterElement);
crate::member_accessors!(ContentFilterElement {
    #[enum(UA_FilterOperator)]
    filterOperator: ua::FilterOperator,
    filterOperands: [ua::ExtensionObject]
});

impl ContentFilterElement {
    #[must_use]
    pub fn with_filter_operator(mut self, filter_operator: ua::FilterOperator) -> Self {
        filter_operator.move_into_raw(&mut self.0.filterOperator);
        self
    }

    #[must_use]
    pub fn with_filter_operands(mut self, filter_operands: &[impl FilterOperand]) -> Self {
        unsafe {
            filter_operands
                .iter()
                .map(FilterOperand::to_extension_object)
                .collect::<ua::Array<_>>()
                .move_into_raw(&mut self.0.filterOperandsSize, &mut self.0.filterOperands);
        }

        self
    }
}
