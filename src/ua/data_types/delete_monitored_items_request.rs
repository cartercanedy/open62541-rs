use crate::ua;

crate::data_type::ua_struct! {
    pub struct DeleteMonitoredItemsRequest {
        #[builder(skip)]
        #[skip(uses = Self::subscription_id)]
        subscriptionId: u32,
        #[builder(skip)]
        monitoredItemIds: [ua::UInt32]
    }
}

impl DeleteMonitoredItemsRequest {
    /// Replaces the monitored item IDs to delete.
    #[must_use]
    pub fn with_monitored_item_ids(mut self, ids: &[ua::MonitoredItemId]) -> Self {
        let ids: ua::Array<ua::UInt32> = ids.iter().map(|id| id.as_id().to_uint32()).collect();
        ids.move_into_raw(
            &mut self.0.monitoredItemIdsSize,
            &mut self.0.monitoredItemIds,
        );
        self
    }

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

#[cfg(test)]
mod tests {
    use crate::{DataType, ua};

    #[test]
    fn typed_monitored_item_ids_keep_uint32_storage_and_empty_sentinel() {
        let ids =
            [1, u32::MAX].map(|id| ua::MonitoredItemId::new(ua::IntegerId::from_u32(id).unwrap()));
        let request = ua::DeleteMonitoredItemsRequest::init().with_monitored_item_ids(&ids);
        assert_eq!(
            request.monitored_item_ids().unwrap(),
            &[ua::UInt32::new(1), ua::UInt32::new(u32::MAX)]
        );

        let request = request.with_monitored_item_ids(&ids[1..]);
        assert_eq!(
            request.monitored_item_ids().unwrap(),
            &[ua::UInt32::new(u32::MAX)]
        );

        let request = request.with_monitored_item_ids(&[]);
        assert_eq!(request.monitored_item_ids().unwrap(), &[]);
        assert_eq!(request.0.monitoredItemIdsSize, 0);
        assert_eq!(request.0.monitoredItemIds.cast_const().cast(), unsafe {
            open62541_sys::UA_EMPTY_ARRAY_SENTINEL
        });
    }
}
