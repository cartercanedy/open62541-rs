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
fn protocol_enum_accepts_int32_wire_value() {
    let wire = ua::Variant::scalar(ua::Int32::new(1));
    assert_eq!(
        wire.as_scalar::<ua::NodeClass>(),
        Some(&ua::NodeClass::OBJECT)
    );
    assert_eq!(
        wire.to_scalar::<ua::NodeClass>(),
        Some(ua::NodeClass::OBJECT)
    );
    assert_eq!(
        wire.try_into_scalar::<ua::NodeClass>().unwrap(),
        ua::NodeClass::OBJECT
    );
    assert!(
        ua::Variant::scalar(ua::UInt32::new(1))
            .into_scalar::<ua::NodeClass>()
            .is_none()
    );
    assert!(
        ua::Variant::scalar(ua::Int32::new(1))
            .into_scalar::<ua::UInt32>()
            .is_none()
    );
}

#[test]
fn protocol_enum_preserves_unknown_high_bits() {
    let value = ua::Variant::scalar(ua::Int32::new(-1))
        .try_into_scalar::<ua::NodeClass>()
        .unwrap();
    // SAFETY: The integer-backed raw conversion has no extra memory-safety preconditions.
    assert_eq!(unsafe { value.as_u32_unchecked() }, u32::MAX);
    // SAFETY: Every bit pattern is representable by the integer-backed enum wrapper.
    let reconstructed = unsafe { ua::NodeClass::from_u32_unchecked(u32::MAX) };
    assert_eq!(value, reconstructed);
}

#[test]
#[expect(deprecated, reason = "verify the old public helpers remain compatible")]
fn extension_object_encoding_raw_helpers_remain_compatible() {
    let raw = open62541_sys::UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_DECODED;
    let old = ua::ExtensionObjectEncoding::from_ua(&raw);
    // SAFETY: This integer-backed conversion has no additional memory-safety preconditions.
    let new = unsafe { ua::ExtensionObjectEncoding::from_ua_unchecked(&raw) };
    assert_eq!(old, new);
    assert_eq!(old.as_u32(), unsafe { new.as_u32_unchecked() });
    assert!(new.is_decoded());
}

#[test]
fn helper_enums_share_uint32_storage_and_extraction() {
    fn check<T: DataType<Inner = u32> + Copy + PartialEq + std::fmt::Display>(value: T) {
        assert_eq!(T::data_type(), ua::UInt32::data_type());
        assert!(!std::mem::needs_drop::<T>());

        let bits = value.into_raw();
        let wire = ua::Variant::scalar(ua::UInt32::new(bits));
        assert_eq!(wire.as_scalar::<T>(), Some(&value));
        assert_eq!(wire.into_scalar::<T>(), Some(value));
        assert_eq!(value.to_string(), bits.to_string());
        assert!(
            ua::Variant::scalar(ua::Int32::new(-1))
                .as_scalar::<T>()
                .is_none()
        );
    }

    // SAFETY: Every integer bit pattern is valid for the helper representations.
    const ATTRIBUTE: ua::AttributeId = unsafe { ua::AttributeId::from_u32_unchecked(u32::MAX) };
    // SAFETY: Every integer bit pattern is valid for the helper representations.
    const NODE_TYPE: ua::NodeIdType = unsafe { ua::NodeIdType::from_u32_unchecked(u32::MAX) };
    // SAFETY: Every integer bit pattern is valid for the helper representations.
    const ENCODING: ua::ExtensionObjectEncoding =
        unsafe { ua::ExtensionObjectEncoding::from_u32_unchecked(u32::MAX) };

    check(ATTRIBUTE);
    check(NODE_TYPE);
    check(ENCODING);

    let read = ua::ReadValueId::init().with_attribute_id(ATTRIBUTE);
    assert_eq!(read.attribute_id(), ATTRIBUTE);
    assert!(ua::NodeId::numeric(0, 42).identifier_type().is_numeric());
    assert!(ua::ExtensionObject::init().encoding().is_encoded_nobody());
    assert_eq!(ua::AttributeId::NODEID.to_string(), "NODEID");
    assert_eq!(ua::NodeIdType::NUMERIC.to_string(), "NUMERIC");
    assert_eq!(ua::ExtensionObjectEncoding::DECODED.to_string(), "DECODED");
}

#[test]
fn protocol_enum_arrays_accept_int32_elements() {
    let wire = ua::Variant::array(ua::Array::from_slice(&[
        ua::Int32::new(1),
        ua::Int32::new(2),
    ]));
    let values = wire.to_array::<ua::NodeClass>().unwrap();
    assert_eq!(values[0], ua::NodeClass::OBJECT);
    assert_eq!(values[1], ua::NodeClass::VARIABLE);
    assert!(wire.to_array::<ua::UInt32>().is_none());
}

#[test]
fn continuation_point_accepts_owned_byte_string() {
    let wire = ua::Variant::scalar(ua::ByteString::new(&[1, 2, 3]));
    let native = wire.to_scalar::<ua::ContinuationPoint>().unwrap();
    assert_eq!(wire.as_scalar::<ua::ContinuationPoint>(), Some(&native));
    assert_eq!(
        wire.try_into_scalar::<ua::ContinuationPoint>().unwrap(),
        native
    );
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
fn protocol_enums_use_their_native_descriptors() {
    fn check<T: DataType>(index: u32) {
        // SAFETY: UA_TYPES is a static array and each index comes from the generated bindings.
        let types = unsafe { (&raw const open62541_sys::UA_TYPES).as_ref().unwrap() };
        let expected = types
            .get(usize::try_from(index).unwrap())
            .expect("generated descriptor index");
        assert!(std::ptr::eq(T::data_type(), expected));
    }
    check::<ua::ApplicationType>(open62541_sys::UA_TYPES_APPLICATIONTYPE);
    check::<ua::DataChangeTrigger>(open62541_sys::UA_TYPES_DATACHANGETRIGGER);
    check::<ua::FilterOperator>(open62541_sys::UA_TYPES_FILTEROPERATOR);
    check::<ua::MessageSecurityMode>(open62541_sys::UA_TYPES_MESSAGESECURITYMODE);
    check::<ua::UserTokenType>(open62541_sys::UA_TYPES_USERTOKENTYPE);
}

#[test]
fn consuming_scalar_reports_type_mismatch() {
    let value = ua::Variant::scalar(ua::UInt32::new(42));
    assert_eq!(
        value
            .clone()
            .try_into_scalar::<ua::UInt32>()
            .unwrap()
            .value(),
        42
    );
    assert_eq!(
        value
            .try_into_scalar::<ua::Int32>()
            .unwrap_err()
            .status_code(),
        ua::StatusCode::BADTYPEMISMATCH,
    );
    assert_eq!(
        ua::Variant::init()
            .try_into_scalar::<ua::UInt32>()
            .unwrap_err()
            .status_code(),
        ua::StatusCode::BADTYPEMISMATCH,
    );
}

#[test]
fn consuming_borrowed_scalar_does_not_modify_source() {
    let mut source = 42_u32;
    let mut raw = open62541_sys::UA_Variant::default();
    unsafe {
        open62541_sys::UA_Variant_setScalar(
            &raw mut raw,
            std::ptr::from_mut(&mut source).cast(),
            ua::UInt32::data_type(),
        );
    }
    raw.storageType = open62541_sys::UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
    // SAFETY: NODELETE leaves the stack-owned source alive and owned by this test.
    let value = unsafe { ua::Variant::from_raw(raw) };
    assert_eq!(value.try_into_scalar::<ua::UInt32>().unwrap().value(), 42);
    assert_eq!(source, 42);
}

#[test]
fn owned_scalar_replacement_reuses_storage_and_leaves_borrowed_storage_alone() {
    let value = ua::Variant::scalar(ua::String::new("before").unwrap());
    let scalar_pointer = std::ptr::from_ref(value.as_scalar::<ua::String>().unwrap());
    let replacement = ua::String::new("after").unwrap();
    let payload_pointer = replacement.as_bytes().unwrap().as_ptr();
    let value = value.with_scalar(replacement);
    let scalar = value.as_scalar::<ua::String>().unwrap();
    assert_eq!(std::ptr::from_ref(scalar), scalar_pointer);
    assert_eq!(scalar.as_bytes().unwrap().as_ptr(), payload_pointer);

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
fn empty_and_undefined_arrays_keep_their_existing_shapes() {
    let empty = ua::Variant::array(ua::Array::<ua::Int32>::new(0));
    assert!(empty.as_scalar::<ua::NodeClass>().is_none());
    assert!(empty.to_array::<ua::NodeClass>().is_some());
    assert!(empty.to_array::<ua::UInt32>().is_none());

    let empty_objects = ua::Variant::array(ua::Array::<ua::ExtensionObject>::new(0));
    assert!(empty_objects.to_array::<ua::NodeClass>().is_some());
    let undefined = ua::Variant::init();
    assert!(undefined.to_array::<ua::NodeClass>().is_none());
}

#[test]
fn open_enumeration_has_unchecked_raw_conversions() {
    // SAFETY: Every bit pattern is valid for the integer-backed open enumeration.
    const VALUE: ua::Enumeration = unsafe { ua::Enumeration::from_u32_unchecked(u32::MAX) };
    // SAFETY: Reading the stored bits requires no additional preconditions.
    assert_eq!(unsafe { VALUE.as_u32_unchecked() }, u32::MAX);
    assert_eq!(VALUE.as_u32(), u32::MAX);
    assert_eq!(VALUE.to_string(), u32::MAX.to_string());
    assert_eq!(ua::NodeClass::OBJECT.to_string(), "OBJECT");
    // SAFETY: Open enums retain all integer bit patterns, including unknown values.
    let unknown = unsafe { ua::NodeClass::from_u32_unchecked(u32::MAX) };
    assert_eq!(unknown.to_string(), VALUE.to_string());
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
fn unsigned_helpers_and_flags_keep_unsigned_inspection() {
    // SAFETY: Every u32 bit pattern is valid for the helper.
    let variant = ua::Variant::scalar(unsafe { ua::AttributeId::from_u32_unchecked(u32::MAX) });
    assert_eq!(variant.value_type(), Some(ValueType::UInt32));
    let VariantValue::Scalar(ScalarValue::UInt32(value)) = variant.to_value() else {
        panic!("expected UInt32 helper inspection");
    };
    assert_eq!(value.value(), u32::MAX);
    #[cfg(feature = "serde")]
    assert_eq!(serde_json::to_string(&variant).unwrap(), "4294967295");

    let variant = ua::Variant::scalar(ua::AccessLevelType::CURRENTREAD);
    assert_eq!(variant.value_type(), Some(ValueType::Byte));
    let VariantValue::Scalar(ScalarValue::Byte(value)) = variant.to_value() else {
        panic!("expected Byte flag inspection");
    };
    assert_eq!(value.value(), 1);
    #[cfg(feature = "serde")]
    assert_eq!(serde_json::to_string(&variant).unwrap(), "1");
    assert_eq!(ua::Variant::init().value_type(), None);
}

#[test]
fn consuming_variant_self_preserves_all_owned_shapes() {
    let values = [
        ua::Variant::init(),
        ua::Variant::scalar(ua::String::new("owned").unwrap()),
        ua::Variant::array(ua::Array::from_slice(&[
            ua::UInt32::new(1),
            ua::UInt32::new(2),
        ])),
        ua::Variant::array(ua::Array::<ua::UInt32>::new(0)),
    ];
    for value in values {
        let expected = value.clone();
        let result = value.try_into_scalar::<ua::Variant>().unwrap();
        assert_eq!(result, expected);
    }
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
fn enum_numeric_views_preserve_native_identity_and_borrowed_storage() {
    type Enum = ua::ApplicationType;
    // SAFETY: The open enum retains every 32-bit pattern.
    let make = |bits| unsafe { Enum::from_u32_unchecked(bits) };
    let bits = [0, 1, 0x7fff_ffff, 0x8000_0000, u32::MAX];
    for raw in bits {
        let value = make(raw);
        let variant = ua::Variant::scalar(value.clone());
        let source_id = variant.type_id().unwrap().clone();
        #[cfg(feature = "serde")]
        let expected = i32::from_ne_bytes(raw.to_ne_bytes());
        assert_eq!(variant.value_type(), Some(ValueType::Enumeration));
        let VariantValue::Scalar(ScalarValue::Enumeration(inspected)) = variant.to_value() else {
            panic!("expected Enumeration inspection");
        };
        assert_eq!(inspected.as_u32(), raw);
        assert_eq!(variant.to_scalar::<ua::Enumeration>().unwrap(), inspected);
        assert_eq!(variant.as_scalar::<ua::Enumeration>(), Some(&inspected));
        assert_eq!(
            variant.clone().into_scalar::<ua::Enumeration>(),
            Some(inspected)
        );
        assert_eq!(variant.as_scalar::<Enum>(), Some(&value));
        assert!(variant.as_scalar::<ua::Int32>().is_none());
        assert!(variant.as_scalar::<ua::UInt32>().is_none());
        assert_eq!(variant.type_id(), Some(&source_id));

        #[cfg(feature = "serde")]
        {
            assert_eq!(
                serde_json::to_string(&variant).unwrap(),
                expected.to_string()
            );
            // SAFETY: The borrowed wrapper does not free data; its owner remains alive.
            let mut raw = unsafe { std::ptr::read(variant.as_ptr()) };
            raw.storageType = UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
            let borrowed = unsafe { ua::Variant::from_raw(raw) };
            assert_eq!(
                serde_json::to_string(&borrowed).unwrap(),
                expected.to_string()
            );
            assert_eq!(borrowed.type_id(), Some(&source_id));
        }
    }

    #[cfg(feature = "serde")]
    {
        let values = bits.map(make);
        let variant = ua::Variant::array(ua::Array::from_slice(&values));
        let source_id = variant.type_id().unwrap().clone();
        let expected = "[0,1,2147483647,-2147483648,-1]";
        assert_eq!(serde_json::to_string(&variant).unwrap(), expected);
        assert_eq!(variant.type_id(), Some(&source_id));
        assert_eq!(variant.to_array::<Enum>().unwrap().as_slice(), values);
        // SAFETY: The borrowed wrapper cannot free the owner or its array dimensions.
        let mut raw = unsafe { std::ptr::read(variant.as_ptr()) };
        raw.storageType = UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
        let borrowed = unsafe { ua::Variant::from_raw(raw) };
        assert_eq!(serde_json::to_string(&borrowed).unwrap(), expected);

        let empty = ua::Variant::array(ua::Array::<Enum>::new(0));
        assert_eq!(empty.value_type(), Some(ValueType::Enumeration));
        assert!(empty.to_array::<ua::Enumeration>().unwrap().is_empty());
        assert_eq!(serde_json::to_string(&empty).unwrap(), "[]");
        // SAFETY: This creates the native undefined-array shape with no owned payload.
        let mut raw = empty.into_raw();
        raw.data = std::ptr::null_mut();
        let undefined = unsafe { ua::Variant::from_raw(raw) };
        assert!(serde_json::to_string(&undefined).is_err());
    }
}

#[test]
fn enumeration_category_supports_native_and_wire_extraction() {
    fn check(value: impl DataType, bits: u32) {
        let variant = ua::Variant::scalar(value);
        assert_eq!(variant.value_type(), Some(ValueType::Enumeration));
        let VariantValue::Scalar(ScalarValue::Enumeration(value)) = variant.to_value() else {
            panic!("expected Enumeration inspection");
        };
        assert_eq!(value.as_u32(), bits);
        assert_eq!(variant.into_scalar::<ua::Enumeration>(), Some(value));
    }
    check(ua::NodeClass::OBJECT, 1);
    // SAFETY: Enumeration retains every integer bit pattern.
    check(
        unsafe { ua::Enumeration::from_u32_unchecked(u32::MAX) },
        u32::MAX,
    );

    let source = ua::Variant::array([ua::NodeClass::OBJECT, ua::NodeClass::VARIABLE].into());
    // SAFETY: The source outlives this NODELETE view, which cannot free its payload.
    let mut raw = unsafe { std::ptr::read(source.as_ptr()) };
    raw.storageType = UA_VariantStorageType::UA_VARIANT_DATA_NODELETE;
    let borrowed = unsafe { ua::Variant::from_raw(raw) };
    assert_eq!(borrowed.value_type(), Some(ValueType::Enumeration));
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

    let wire = ua::Variant::scalar(ua::Int32::new(-1));
    assert_eq!(wire.value_type(), Some(ValueType::Int32));
    assert!(matches!(
        wire.to_value(),
        VariantValue::Scalar(ScalarValue::Int32(_))
    ));
    assert_eq!(
        wire.to_scalar::<ua::Enumeration>().unwrap().as_u32(),
        u32::MAX
    );
    assert!(
        ua::Variant::scalar(ua::UInt32::new(1))
            .to_scalar::<ua::Enumeration>()
            .is_none()
    );
    assert!(!ua::Enumeration::accepts_variant_type(std::ptr::null()));
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
