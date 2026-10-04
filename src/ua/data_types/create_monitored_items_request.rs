use crate::ua;

crate::data_type::ua_struct! {
    pub struct CreateMonitoredItemsRequest {
        itemsToCreate: [ua::MonitoredItemCreateRequest],
        timestampsToReturn: &ua::TimestampsToReturn,
        #[builder(skip)]
        #[skip(uses = Self::subscription_id)]
        subscriptionId: u32
    }
}

impl CreateMonitoredItemsRequest {
    #[must_use]
    pub const fn with_subscription_id(mut self, subscription_id: ua::SubscriptionId) -> Self {
        self.0.subscriptionId = subscription_id.as_id().as_u32();
        self
    }

    #[must_use]
    pub const fn subscription_id(&self) -> Option<ua::SubscriptionId> {
        if let Some(id) = ua::IntegerId::from_u32(self.0.subscriptionId) {
            Some(ua::SubscriptionId::new(id))
        } else {
            None
        }
    }
}
