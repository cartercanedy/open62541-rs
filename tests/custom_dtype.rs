use std::ptr;

use open62541::{DataType, ScalarValue, ValueType, VariantValue, ua};
use open62541_sys::UA_VariantStorageType;

#[derive(Debug, Clone)]
#[repr(transparent)]
struct CustomCounter(ua::UInt32);

// SAFETY: CustomCounter is transparent over UInt32 and uses its native storage and descriptor.
unsafe impl DataType for CustomCounter {
    type Inner = open62541_sys::UA_UInt32;

    fn data_type() -> *const open62541_sys::UA_DataType {
        ua::UInt32::data_type()
    }

    unsafe fn from_raw(value: Self::Inner) -> Self {
        // SAFETY: The caller transfers a valid UInt32 value.
        Self(unsafe { ua::UInt32::from_raw(value) })
    }

    fn into_raw(self) -> Self::Inner {
        self.0.into_raw()
    }

    fn try_from_variant(value: ua::Variant) -> open62541::Result<Self> {
        Self::try_clone_from_variant(&value)
    }

    fn try_clone_from_variant(value: &ua::Variant) -> open62541::Result<Self> {
        if let Some(native) = value.as_scalar::<ua::UInt32>() {
            return Ok(Self(*native));
        }

        let wire = value
            .as_scalar::<ua::Int32>()
            .ok_or(open62541::Error::Server(ua::StatusCode::BADTYPEMISMATCH))?
            .value();
        let counter =
            u32::try_from(wire).map_err(|_| open62541::Error::Internal("negative counter"))?;

        Ok(Self(ua::UInt32::new(counter)))
    }
}

#[test]
fn downstream_type_can_supply_an_owned_conversion_without_a_borrow() {
    let wire = ua::Variant::scalar(ua::Int32::new(42));

    assert!(wire.as_scalar::<CustomCounter>().is_none());
    assert_eq!(wire.to_scalar::<CustomCounter>().unwrap().0.value(), 42);
    assert_eq!(
        wire.try_into_scalar::<CustomCounter>().unwrap().0.value(),
        42
    );

    let native = ua::Variant::scalar(ua::UInt32::new(7));

    assert_eq!(
        native.try_into_scalar::<CustomCounter>().unwrap().0.value(),
        7
    );

    let invalid = ua::Variant::scalar(ua::Int32::new(-1));

    assert!(invalid.to_scalar::<CustomCounter>().is_none());
    assert!(matches!(
        invalid.try_into_scalar::<CustomCounter>(),
        Err(open62541::Error::Internal("negative counter"))
    ));
}

#[test]
fn borrowed_allocating_wire_value_is_not_moved_or_freed() {
    let source = ua::ByteString::new(&[1, 2, 3]);
    let mut raw = open62541_sys::UA_Variant::default();
    unsafe {
        open62541_sys::UA_Variant_setScalar(
            &raw mut raw,
            source.as_ptr().cast_mut().cast(),
            ua::ByteString::data_type(),
        );
    }

    raw.storageType = open62541_sys::UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;

    // SAFETY: NODELETE leaves the source owned by this test for the lifetime of the variant.
    let value = unsafe { ua::Variant::from_raw(raw) };
    let decoded = value.try_into_scalar::<ua::ContinuationPoint>().unwrap();

    assert_eq!(source.as_bytes(), Some(&[1, 2, 3][..]));

    drop(decoded);

    assert_eq!(source.as_bytes(), Some(&[1, 2, 3][..]));
}

#[test]
fn scalar_replacement_preserves_borrowed_storage() {
    let mut source = 7_u32;
    let mut raw = open62541_sys::UA_Variant::default();
    // SAFETY: The stack scalar stays alive and NODELETE prevents transferring its ownership.
    unsafe {
        open62541_sys::UA_Variant_setScalar(
            &raw mut raw,
            std::ptr::from_mut(&mut source).cast(),
            ua::UInt32::data_type(),
        );
    }

    raw.storageType = open62541_sys::UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;

    // SAFETY: The scalar remains valid until this borrowed Variant is consumed below.
    let value = unsafe { ua::Variant::from_raw(raw) }.with_scalar(ua::UInt32::new(42));

    assert_eq!(value.as_scalar::<ua::UInt32>().unwrap().value(), 42);
    assert_eq!(source, 7);
}

#[test]
fn enum_policy_does_not_coerce_unrelated_types() {
    let wire = ua::Variant::scalar(ua::Int32::new(0));

    assert_eq!(
        wire.to_scalar::<ua::ApplicationType>(),
        Some(ua::ApplicationType::SERVER)
    );
    assert!(wire.to_scalar::<ua::UInt32>().is_none());
    assert!(wire.to_scalar::<ua::AttributeId>().is_none());

    let named = ua::Variant::scalar(ua::NodeClass::OBJECT);

    assert!(named.to_scalar::<ua::ApplicationType>().is_none());

    let unsigned = ua::Variant::scalar(ua::UInt32::new(0));

    assert!(unsigned.to_scalar::<ua::ApplicationType>().is_none());
}

#[test]
fn consuming_borrowed_variant_self_clones_allocating_contents() {
    let source = ua::Variant::scalar(ua::String::new("borrowed").unwrap());

    // SAFETY: NODELETE prevents this wrapper from freeing source's live contents.
    let mut raw = unsafe { std::ptr::read(source.as_ptr()) };
    raw.storageType = UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
    let borrowed = unsafe { ua::Variant::from_raw(raw) };
    let result = borrowed.try_into_scalar::<ua::Variant>().unwrap();

    assert_eq!(result, source);

    drop(source);

    assert_eq!(
        result.as_scalar::<ua::String>().unwrap().as_str(),
        Some("borrowed")
    );
}

#[test]
fn enum_inspection_and_serde_preserve_native_identity() {
    // SAFETY: The open enum accepts every integer bit pattern.
    let value = unsafe { ua::ApplicationType::from_u32_unchecked(u32::MAX) };
    let variant = ua::Variant::scalar(value.clone());
    let source_id = variant.type_id().unwrap().clone();

    assert_eq!(variant.value_type(), Some(ValueType::Enumeration));

    let VariantValue::Scalar(ScalarValue::Enumeration(inspected)) = variant.to_value() else {
        panic!("expected Enumeration inspection");
    };

    assert_eq!(inspected.as_u32(), u32::MAX);
    assert_eq!(variant.as_scalar::<ua::Enumeration>(), Some(&inspected));
    assert_eq!(variant.as_scalar::<ua::ApplicationType>(), Some(&value));
    assert!(variant.as_scalar::<ua::Int32>().is_none());
    assert!(variant.as_scalar::<ua::UInt32>().is_none());

    #[cfg(feature = "serde")]
    {
        // SAFETY: The owner outlives this view; NODELETE prevents freeing its payload.
        let mut raw = unsafe { ptr::read(variant.as_ptr()) };
        raw.storageType = UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
        let borrowed = unsafe { ua::Variant::from_raw(raw) };

        assert_eq!(serde_json::to_string(&borrowed).unwrap(), "-1");
        assert_eq!(borrowed.type_id(), Some(&source_id));

        let array = ua::Variant::array([value].into());

        assert_eq!(serde_json::to_string(&array).unwrap(), "[-1]");
    }

    assert_eq!(variant.type_id(), Some(&source_id));
}

#[test]
fn enumeration_array_extraction_clones_borrowed_storage() {
    let source = ua::Variant::array([ua::NodeClass::OBJECT, ua::NodeClass::VARIABLE].into());

    // SAFETY: The source outlives the NODELETE view, which cannot free its payload.
    let mut raw = unsafe { ptr::read(source.as_ptr()) };
    raw.storageType = UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
    let borrowed = unsafe { ua::Variant::from_raw(raw) };

    assert!(borrowed.to_array::<ua::Int32>().is_none());

    let array = borrowed.to_array::<ua::Enumeration>().unwrap();

    drop(borrowed);
    drop(source);

    assert_eq!(
        array
            .iter()
            .map(ua::Enumeration::as_u32)
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn enumeration_extraction_requires_enum_storage_not_just_its_node_id() {
    // SAFETY: Copying the builtin descriptor does not transfer its static members or numeric IDs.
    let mut descriptor = unsafe { std::ptr::read(ua::Byte::data_type()) };
    descriptor.typeId = ua::NodeId::ns0(open62541_sys::UA_NS0ID_ENUMERATION).into_raw();
    let mut bytes = [1_u8, 2];

    for length in [0, bytes.len()] {
        let raw = open62541_sys::UA_Variant {
            type_: &raw const descriptor,
            storageType: UA_VariantStorageType::UA_VARIANT_DATA_NODELETE,
            arrayLength: length,
            data: bytes.as_mut_ptr().cast(),
            ..Default::default()
        };
        // SAFETY: Byte metadata and NODELETE storage stay alive throughout this view.
        let value = unsafe { ua::Variant::from_raw(raw) };

        assert!(value.as_scalar::<ua::Enumeration>().is_none());
        assert!(value.to_scalar::<ua::Enumeration>().is_none());
        assert!(value.to_array::<ua::Enumeration>().is_none());
    }
}

#[test]
fn native_masks_retain_enum_identity_and_accept_int32_storage() {
    fn check<T: DataType<Inner = u32> + Copy + PartialEq>(index: u32) {
        // SAFETY: The generated index belongs to the native static descriptor array.
        let expected = unsafe { (&raw const open62541_sys::UA_TYPES).as_ref().unwrap() }
            .get(usize::try_from(index).unwrap())
            .expect("native mask descriptor exists");

        assert!(ptr::eq(T::data_type(), expected));

        // SAFETY: These integer-backed masks accept every u32 bit pattern.
        let value = unsafe { T::from_raw(u32::MAX) };
        let native = ua::Variant::scalar(value);

        assert_eq!(
            native.type_id(),
            Some(ua::NodeId::raw_ref(&expected.typeId))
        );
        assert_eq!(native.value_type(), Some(ValueType::Enumeration));
        assert_eq!(
            native.to_scalar::<ua::Enumeration>().unwrap().as_u32(),
            u32::MAX
        );
        assert!(native.to_scalar::<ua::UInt32>().is_none());

        let decoded = ua::Variant::scalar(ua::Int32::new(-1));

        assert_eq!(decoded.as_scalar::<T>(), Some(&value));
        assert_eq!(decoded.into_scalar::<T>(), Some(value));
        assert!(
            ua::Variant::scalar(ua::UInt32::new(1))
                .to_scalar::<T>()
                .is_none()
        );

        let wire = ua::Variant::array([ua::Int32::new(-1), ua::Int32::new(-1)].into());

        assert_eq!(wire.to_array::<T>().unwrap().as_slice(), [value, value]);
    }

    check::<ua::TrustListMasks>(open62541_sys::UA_TYPES_TRUSTLISTMASKS);
    check::<ua::BrowseResultMask>(open62541_sys::UA_TYPES_BROWSERESULTMASK);
    check::<ua::SpecifiedAttributes>(open62541_sys::UA_TYPES_NODEATTRIBUTESMASK);
}
