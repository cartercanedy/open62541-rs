use crate::ua;

crate::data_type!(DeleteSubscriptionsRequest);
crate::member_accessors!(DeleteSubscriptionsRequest {
    subscriptionIds: [ua::UInt32],
    requestHeader: &mut ua::RequestHeader
});

impl DeleteSubscriptionsRequest {
    #[must_use]
    pub fn with_subscription_ids(mut self, subscription_ids: &[ua::SubscriptionId]) -> Self {
        unsafe {
            subscription_ids
                .iter()
                .map(|subscription_id| subscription_id.as_id().to_uint32())
                .collect::<ua::Array<_>>()
                .move_into_raw(&mut self.0.subscriptionIdsSize, &mut self.0.subscriptionIds);
        }

        self
    }
}
