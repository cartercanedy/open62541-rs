use crate::ua;

crate::data_type::ua_struct! {
    pub struct DeleteSubscriptionsRequest {
        #[builder(skip)]
        subscriptionIds: [ua::UInt32],
        requestHeader: &mut ua::RequestHeader
    }
}

impl DeleteSubscriptionsRequest {
    /// Replaces the subscription IDs to delete.
    #[must_use]
    pub fn with_subscription_ids(mut self, ids: &[ua::SubscriptionId]) -> Self {
        let ids: ua::Array<ua::UInt32> = ids.iter().map(|id| id.as_id().to_uint32()).collect();
        ids.move_into_raw(&mut self.0.subscriptionIdsSize, &mut self.0.subscriptionIds);
        self
    }
}
