use crate::{DataType as _, ua};

crate::data_type::ua_struct! {
    pub struct BrowseResult {
        #[from_inner]
        statusCode: ua::StatusCode,
        references: [ua::ReferenceDescription],

        #[skip(uses = BrowseResult::continuation_point)]
        continuationPoint: &ua::ContinuationPoint
    }
}

impl BrowseResult {
    /// Gets continuation point.
    ///
    /// Browse results include a continuation point when not all references could be returned. Pass
    /// it to [`AsyncClient::browse_next()`] to request the remaining references.
    ///
    /// [`AsyncClient::browse_next()`]: crate::AsyncClient::browse_next
    #[must_use]
    pub fn continuation_point(&self) -> Option<ua::ContinuationPoint> {
        ua::ContinuationPoint::new(ua::ByteString::raw_ref(&self.0.continuationPoint).clone())
    }

    /// Moves out the continuation point, leaving it unset in this result.
    #[must_use]
    pub fn take_continuation_point(&mut self) -> Option<ua::ContinuationPoint> {
        // SAFETY: The result exclusively owns this initialized ByteString field.
        ua::ContinuationPoint::new(unsafe {
            ua::ByteString::take_raw(&mut self.0.continuationPoint)
        })
    }
}
