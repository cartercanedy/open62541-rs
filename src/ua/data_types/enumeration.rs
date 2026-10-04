// Enumeration contains values from any native protocol enum. The normal declaration
// arm fixes conversion to one named descriptor or Int32, so @protocol lets us supply
// broader Variant conversion hooks here while retaining shared storage, ordering,
// raw conversions, and numeric Serde generation.
crate::data_type::ua_enum! {
    @protocol Enumeration {};
    fn accepts_variant_type(data_type: *const open62541_sys::UA_DataType) -> bool {
        data_type == Self::data_type()
            || data_type == <crate::ua::Int32 as crate::DataType>::data_type()
    }

    fn borrow_from_variant(value: &crate::ua::Variant) -> Option<&Self> {
        if !value.has_enum_type() {
            return value.as_scalar_matching::<Self>();
        }
        if !value.is_scalar() {
            return None;
        }
        // SAFETY: A live scalar ENUM has initialized 32-bit integer storage, and
        // Enumeration accepts every bit pattern. The reference borrows the Variant.
        let raw = unsafe { &*value.as_ptr() };
        Some(Self::raw_ref(unsafe { &*raw.data.cast::<Self::Inner>() }))
    }

    fn try_from_variant(value: crate::ua::Variant) -> crate::Result<Self> {
        // Enumeration is pointer-free; cloning copies the bits before the owner drops.
        Self::try_clone_from_variant(&value)
    }

    fn array_from_variant(value: &crate::ua::Variant) -> Option<crate::ua::Array<Self>> {
        if !value.has_enum_type() {
            return value.to_array_matching::<Self>();
        }
        if value.is_scalar() {
            return None;
        }
        // SAFETY: Native ENUM arrays have Enumeration's layout and integer validity.
        let raw = unsafe { &*value.as_ptr() };
        crate::ua::Array::from_raw_parts(raw.arrayLength, raw.data.cast::<Self::Inner>())
    }
}

impl Enumeration {
    /// Extracts raw enum value.
    #[must_use]
    pub const fn as_u32(&self) -> u32 {
        u32::from_ne_bytes(self.0.0.to_ne_bytes())
    }
}
