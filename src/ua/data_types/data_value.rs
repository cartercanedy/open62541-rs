use crate::{DataType, DataTypeExt, ua};

crate::data_type::ua_struct! {
    pub struct DataValue {
        #[builder(skip)]
        #[skip(uses = Self::value)]
        value: &ua::Variant,
        #[builder(skip)]
        #[skip(uses = Self::server_timestamp)]
        serverTimestamp: i64,
        #[builder(skip)]
        #[skip(uses = Self::source_timestamp)]
        sourceTimestamp: i64,
        #[builder(skip)]
        #[skip(uses = Self::server_picoseconds)]
        serverPicoseconds: u16,
        #[builder(skip)]
        #[skip(uses = Self::source_picoseconds)]
        sourcePicoseconds: u16,
        #[builder(skip)]
        #[skip(uses = Self::status)]
        status: u32
    }
}

impl DataValue {
    #[must_use]
    pub fn new(value: ua::Variant) -> Self {
        let mut inner = ua::DataValue::init();
        // SAFETY: The wrapper exclusively owns this initialized native field.
        unsafe { value.move_into_raw(&mut inner.0.value) };
        inner.0.set_hasValue(true);
        debug_assert!(!inner.0.hasStatus());
        debug_assert!(!inner.0.hasSourceTimestamp());
        debug_assert!(!inner.0.hasServerTimestamp());
        debug_assert!(!inner.0.hasSourcePicoseconds());
        debug_assert!(!inner.0.hasServerPicoseconds());
        inner
    }

    #[must_use]
    pub fn with_value(mut self, value: ua::Variant) -> Self {
        // SAFETY: The wrapper exclusively owns this initialized native field.
        unsafe { value.move_into_raw(&mut self.0.value) };
        self.0.set_hasValue(true);
        self
    }

    #[must_use]
    pub fn with_source_timestamp(mut self, source_timestamp: ua::DateTime) -> Self {
        self.0.sourceTimestamp = source_timestamp.into_raw();
        self.0.set_hasSourceTimestamp(true);
        self
    }

    #[must_use]
    pub fn with_server_timestamp(mut self, server_timestamp: ua::DateTime) -> Self {
        self.0.serverTimestamp = server_timestamp.into_raw();
        self.0.set_hasServerTimestamp(true);
        self
    }

    #[must_use]
    pub fn with_source_picoseconds(mut self, source_picoseconds: u16) -> Self {
        self.0.sourcePicoseconds = source_picoseconds;
        self.0.set_hasSourcePicoseconds(true);
        self
    }

    #[must_use]
    pub fn with_server_picoseconds(mut self, server_picoseconds: u16) -> Self {
        self.0.serverPicoseconds = server_picoseconds;
        self.0.set_hasServerPicoseconds(true);
        self
    }

    #[must_use]
    pub fn with_status(mut self, status: ua::StatusCode) -> Self {
        self.0.status = status.into_raw();
        self.0.set_hasStatus(true);
        self
    }

    /// Gets value.
    ///
    /// This returns the value as [`ua::Variant`] if it is set. Returns `None` when the `DataValue`
    /// holds no value.
    #[must_use]
    pub fn value(&self) -> Option<&ua::Variant> {
        self.0
            .hasValue()
            .then(|| ua::Variant::raw_ref(&self.0.value))
    }

    #[must_use]
    pub(crate) fn into_value(mut self) -> Option<ua::Variant> {
        self.0.hasValue().then(|| {
            // SAFETY: The DataValue owns the initialized payload, including any no-delete tag.
            unsafe { DataType::take_raw(&mut self.0.value) }
        })
    }

    #[must_use]
    pub fn source_timestamp(&self) -> Option<&ua::DateTime> {
        self.0
            .hasSourceTimestamp()
            .then(|| ua::DateTime::raw_ref(&self.0.sourceTimestamp))
    }

    #[must_use]
    pub fn server_timestamp(&self) -> Option<&ua::DateTime> {
        self.0
            .hasServerTimestamp()
            .then(|| ua::DateTime::raw_ref(&self.0.serverTimestamp))
    }

    #[must_use]
    pub fn source_picoseconds(&self) -> Option<u16> {
        self.0
            .hasSourcePicoseconds()
            .then_some(self.0.sourcePicoseconds)
    }

    #[must_use]
    pub fn server_picoseconds(&self) -> Option<u16> {
        self.0
            .hasServerPicoseconds()
            .then_some(self.0.serverPicoseconds)
    }

    #[must_use]
    pub fn status(&self) -> Option<ua::StatusCode> {
        self.0
            .hasStatus()
            .then(|| ua::StatusCode::new(self.0.status))
    }

    /// Casts to specific value type.
    ///
    /// This adjusts the target type of `self`, casting the inner value to the specified data type
    /// when read with [`Self::value()`]. Use this when the type can be deduced from circumstances
    /// and typed data values should be returned for convenience.
    ///
    /// [`Self::value()`]: crate::DataValue::value
    #[must_use]
    pub const fn cast<T: DataTypeExt>(self) -> crate::DataValue<T> {
        crate::DataValue::new(self)
    }
}
