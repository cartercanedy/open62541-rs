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

#[cfg(test)]
mod tests {
    use crate::{DataType, ua};

    #[test]
    fn typed_subscription_ids_keep_uint32_storage_and_empty_sentinel() {
        let ids =
            [1, u32::MAX].map(|id| ua::SubscriptionId::new(ua::IntegerId::from_u32(id).unwrap()));
        let request = ua::DeleteSubscriptionsRequest::init().with_subscription_ids(&ids);
        assert_eq!(
            request.subscription_ids().unwrap(),
            &[ua::UInt32::new(1), ua::UInt32::new(u32::MAX)]
        );

        let request = request.with_subscription_ids(&ids[1..]);
        assert_eq!(
            request.subscription_ids().unwrap(),
            &[ua::UInt32::new(u32::MAX)]
        );

        let request = request.with_subscription_ids(&[]);
        assert_eq!(request.subscription_ids().unwrap(), &[]);
        assert_eq!(request.0.subscriptionIdsSize, 0);
        assert_eq!(request.0.subscriptionIds.cast_const().cast(), unsafe {
            open62541_sys::UA_EMPTY_ARRAY_SENTINEL
        });
    }
}
