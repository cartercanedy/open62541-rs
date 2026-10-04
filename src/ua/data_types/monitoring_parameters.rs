use std::time::Duration;

use crate::ua;

crate::data_type::ua_struct! {
    pub struct MonitoringParameters {
        filter: &ua::ExtensionObject,
        #[builder(skip)]
        #[skip(uses = Self::sampling_interval)]
        samplingInterval: f64,
        queueSize: u32,
        /// When the queue is full, `true` discards the oldest notification.
        /// `false` replaces the newest queued notification.
        discardOldest: bool,
        /// The native client assigns its own handle when creating a monitored item.
        clientHandle: u32
    }
}

impl MonitoringParameters {
    /// Sets sampling interval.
    ///
    /// The value `Some(Duration::ZERO)` indicates that the server should use the fastest practical
    /// rate.
    ///
    /// The value `None` (-1) indicates that the default sampling interval defined by the publishing
    /// interval of the subscription is requested.
    #[must_use]
    pub fn with_sampling_interval(mut self, sampling_interval: Option<Duration>) -> Self {
        self.0.samplingInterval = if let Some(sampling_interval) = sampling_interval {
            sampling_interval.as_secs_f64() * 1e3
        } else {
            -1.0
        };
        self
    }

    #[must_use]
    pub fn sampling_interval(&self) -> Option<Duration> {
        Duration::try_from_secs_f64(self.0.samplingInterval / 1e3).ok()
    }
}
