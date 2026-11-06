use open62541::{Attributes, DataType, ua};
use open62541_sys::UA_NodeAttributesMask;

const fn mask(value: &UA_NodeAttributesMask) -> u32 {
    u32::from_ne_bytes(value.0.to_ne_bytes())
}

fn specified<T: Attributes>(value: &T) -> u32 {
    value.as_node_attributes().specified_attributes()
}

#[test]
fn builtin_scalars_extract_as_masks() {
    fn check<T: DataType + PartialEq>(value: impl DataType, expected: T) {
        let variant = ua::Variant::scalar(value);

        assert_eq!(variant.as_scalar::<T>(), Some(&expected));
        assert_eq!(variant.into_scalar::<T>(), Some(expected));
    }

    check(ua::Byte::new(1), ua::AccessLevelType::CURRENTREAD);
    check(
        ua::UInt16::new(1),
        ua::AccessRestrictionType::SIGNINGREQUIRED,
    );
    check(ua::UInt32::new(1), ua::AttributeWriteMask::ACCESSLEVEL);
}

#[test]
fn const_mask_operations_preserve_byte_bits() {
    const READ: ua::AccessLevelType = ua::AccessLevelType::CURRENTREAD;
    const WRITE: ua::AccessLevelType = ua::AccessLevelType::CURRENTWRITE;
    // SAFETY: Every byte pattern is valid mask storage.
    const UNKNOWN: ua::AccessLevelType = unsafe { ua::AccessLevelType::from_u8_unchecked(0x80) };
    const SET: ua::AccessLevelType = UNKNOWN.set(READ);
    const UNSET: ua::AccessLevelType = SET.unset(READ);
    const CLEARED: ua::AccessLevelType = SET.clear();

    assert_eq!(SET.into_raw(), 0x81);
    assert_eq!(UNSET.into_raw(), 0x80);
    assert_eq!(CLEARED.into_raw(), 0);
    assert_eq!((READ | WRITE).into_raw(), 0x03);
    assert_eq!((SET & READ).into_raw(), 0x01);
    assert_eq!((!READ).into_raw(), 0xfe);
}

#[test]
fn variable_assignments_preserve_presence_for_default_and_replacement_values() {
    let expected = 0x80053;
    let value = ua::VariableAttributes::init();

    assert_eq!(specified(&value), 0);

    let mut value = value
        .with_data_type(ua::NodeId::ns0(open62541_sys::UA_NS0ID_STRING))
        .with_value_rank(-1)
        .with_access_level(ua::AccessLevelType::CURRENTREAD)
        .with_array_dimensions([ua::UInt32::new(2)].into())
        .with_display_name(ua::LocalizedText::new("", "name").unwrap());

    assert_eq!(specified(&value), expected);

    // SAFETY: The initialized native value is exclusively borrowed.
    unsafe { (*value.as_mut_ptr()).specifiedAttributes = 0x8000_0000 };

    let value = value
        .with_data_type(ua::NodeId::init())
        .with_value_rank(0)
        .with_access_level(ua::AccessLevelType::NONE)
        .with_array_dimensions([].into())
        .with_display_name(ua::LocalizedText::init());

    assert_eq!(specified(&value), expected | 0x8000_0000);
    assert_eq!(value.value_rank(), 0);
    assert_eq!(value.array_dimensions().unwrap(), []);
}

#[test]
fn method_boolean_assignments_mark_presence_even_when_false() {
    let value = ua::MethodAttributes::init()
        .with_executable(false)
        .with_user_executable(false);

    assert!(!value.executable());
    assert!(!value.user_executable());
    assert_eq!(
        specified(&value),
        mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_EXECUTABLE)
            | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_USEREXECUTABLE)
    );
}

#[test]
fn trust_list_builders_update_only_their_native_selection_bit() {
    use open62541_sys::UA_TrustListMasks;

    type Builder = fn(ua::TrustListDataType, ua::Array<ua::ByteString>) -> ua::TrustListDataType;

    let cases: [(Builder, UA_TrustListMasks); 4] = [
        (
            ua::TrustListDataType::with_trusted_certificates,
            UA_TrustListMasks::UA_TRUSTLISTMASKS_TRUSTEDCERTIFICATES,
        ),
        (
            ua::TrustListDataType::with_trusted_crls,
            UA_TrustListMasks::UA_TRUSTLISTMASKS_TRUSTEDCRLS,
        ),
        (
            ua::TrustListDataType::with_issuer_certificates,
            UA_TrustListMasks::UA_TRUSTLISTMASKS_ISSUERCERTIFICATES,
        ),
        (
            ua::TrustListDataType::with_issuer_crls,
            UA_TrustListMasks::UA_TRUSTLISTMASKS_ISSUERCRLS,
        ),
    ];

    for (build, native) in cases {
        let expected = u32::from_ne_bytes(native.0.to_ne_bytes());
        let value = build(
            ua::TrustListDataType::init(),
            [ua::ByteString::new(&[1])].into(),
        );

        assert_eq!(value.specified_lists(), expected);

        let mut value = value;

        // SAFETY: Exclusively borrowing initialized native scalar storage.
        unsafe { (*value.as_mut_ptr()).specifiedLists |= 0x8000_0000 };

        let value = build(value, [].into());

        assert_eq!(value.specified_lists(), 0x8000_0000);
    }
}
