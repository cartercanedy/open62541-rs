use crate::ua;

crate::data_type::ua_struct! {
    pub struct BrowsePathTarget {
        targetId: &ua::ExpandedNodeId,

        #[builder(skip)]
        #[skip(uses = BrowsePathTarget::remaining_path_index)]
        remainingPathIndex: u32
    }
}

impl BrowsePathTarget {
    /// Returns the index of the first unprocessed element in the [`ua::RelativePath`].
    ///
    /// This returns `None` if all elements were processed.
    ///
    /// # Panics
    ///
    /// The index must be in range of the compilation target's `usize` type. This is mostly a
    /// concern for 16-bit targets where index values larger than 65535 could not be represented.
    #[must_use]
    pub fn remaining_path_index(&self) -> Option<usize> {
        // The maximum index value indicates that all elements were processed.
        if self.0.remainingPathIndex == u32::MAX {
            return None;
        }
        // PANIC: It is not expected that the index is larger than 65535, so this should work even
        // on 16-bit compilation targets. On 32-bit or 64-bit targets, the limit is much higher.
        Some(
            self.0
                .remainingPathIndex
                .try_into()
                .expect("remaining path index should be in range of usize"),
        )
    }
}
