use std::ptr;

use open62541::{Attributes, DataType, ua};
use open62541_sys::{UA_NodeAttributesMask, UA_encodeBinary};

const fn mask(value: &UA_NodeAttributesMask) -> u32 {
    u32::from_ne_bytes(value.0.to_ne_bytes())
}

fn specified<T: Attributes>(value: &T) -> u32 {
    value.as_node_attributes().specified_attributes()
}

fn encoded<T: DataType>(value: &T) -> ua::ByteString {
    let mut bytes = ua::ByteString::init();
    // SAFETY: The source and destination are live values with their matching descriptors.
    let status = unsafe {
        UA_encodeBinary(
            value.as_ptr().cast(),
            T::data_type(),
            bytes.as_mut_ptr(),
            ptr::null_mut(),
        )
    };
    assert_eq!(status, open62541_sys::UA_STATUSCODE_GOOD);
    bytes
}

#[test]
fn variable_assignments_encode_independent_presence_masks() {
    let expected = mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DATATYPE)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_VALUERANK)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ACCESSLEVEL)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DISPLAYNAME);
    assert_eq!(expected, 0x80051);

    for value in [
        ua::VariableAttributes::init(),
        ua::VariableAttributes::default(),
    ] {
        assert_eq!(specified(&value), 0);
        let value = value
            .with_data_type(ua::NodeId::ns0(open62541_sys::UA_NS0ID_STRING))
            .with_value_rank(-1)
            .with_access_level(ua::AccessLevelType::CURRENTREAD)
            .with_display_name(ua::LocalizedText::new("", "name").unwrap());
        assert_eq!(specified(&value), expected);
        assert_eq!(
            encoded(&value).as_bytes().unwrap().get(..4),
            Some([0x51, 0, 8, 0].as_slice())
        );
    }
}

#[test]
fn strict_receiver_uses_only_fields_with_presence_bits() {
    fn received_rank(value: &ua::VariableAttributes) -> Option<i32> {
        let bit = mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_VALUERANK);
        (specified(value) & bit != 0).then(|| value.value_rank())
    }

    fn received_access(value: &ua::VariableAttributes) -> Option<ua::AccessLevelType> {
        let bit = mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ACCESSLEVEL);
        (specified(value) & bit != 0).then(|| value.access_level())
    }

    let value = ua::VariableAttributes::init();
    assert_eq!(received_rank(&value), None);
    assert_eq!(received_access(&value), None);
    let value = value
        .with_value_rank(0)
        .with_access_level(ua::AccessLevelType::NONE);
    assert_eq!(received_rank(&value), Some(0));
    assert_eq!(received_access(&value), Some(ua::AccessLevelType::NONE));
}

#[test]
fn empty_and_default_assignments_remain_specified() {
    let value = ua::VariableAttributes::init()
        .with_data_type(ua::NodeId::init())
        .with_value_rank(0)
        .with_access_level(ua::AccessLevelType::NONE)
        .with_array_dimensions([ua::UInt32::new(2)].into())
        .with_array_dimensions([].into())
        .with_display_name(ua::LocalizedText::init());
    let expected = mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DATATYPE)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_VALUERANK)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ACCESSLEVEL)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_ARRAYDIMENSIONS)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DISPLAYNAME);
    assert_eq!(specified(&value), expected);
    assert_eq!(value.array_dimensions().unwrap(), []);
}

#[test]
fn inherited_display_name_assignments_mark_each_attribute_type() {
    fn check<T: Attributes + Default>() {
        for value in [T::init(), T::default()] {
            assert_eq!(specified(&value), 0);
            let value = value.with_display_name(ua::LocalizedText::init());
            assert_eq!(
                specified(&value),
                mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DISPLAYNAME)
            );
        }
    }

    check::<ua::ObjectAttributes>();
    check::<ua::VariableAttributes>();
    check::<ua::MethodAttributes>();
    check::<ua::ObjectTypeAttributes>();
    check::<ua::VariableTypeAttributes>();
    check::<ua::ReferenceTypeAttributes>();
    check::<ua::DataTypeAttributes>();
    check::<ua::ViewAttributes>();
}

#[test]
fn method_boolean_assignments_mark_presence_even_when_false() {
    const fn configure(value: ua::MethodAttributes, executable: bool) -> ua::MethodAttributes {
        value
            .with_executable(executable)
            .with_user_executable(false)
    }

    let expected = mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_EXECUTABLE)
        | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_USEREXECUTABLE);
    for executable in [false, true] {
        for value in [
            ua::MethodAttributes::init(),
            ua::MethodAttributes::default(),
        ] {
            let value = configure(value, executable);
            assert_eq!(value.executable(), executable);
            assert!(!value.user_executable());
            assert_eq!(specified(&value), expected);
        }
    }
}

#[test]
fn historizing_uses_the_native_mask_value() {
    assert_eq!(
        ua::SpecifiedAttributes::HISTORIZING.into_raw(),
        mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_HISTORIZING),
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

#[test]
fn replacement_assignments_preserve_unrelated_presence_bits() {
    let mut value = ua::VariableAttributes::init();
    // SAFETY: The initialized value is exclusively borrowed while its scalar mask is assigned.
    unsafe { (*value.as_mut_ptr()).specifiedAttributes = 0x8000_0000 };
    let value = value
        .with_value_rank(1)
        .with_value_rank(0)
        .with_display_name(ua::LocalizedText::new("", "old").unwrap())
        .with_display_name(ua::LocalizedText::init());
    assert_eq!(value.value_rank(), 0);
    assert_eq!(
        specified(&value),
        0x8000_0000
            | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_VALUERANK)
            | mask(&UA_NodeAttributesMask::UA_NODEATTRIBUTESMASK_DISPLAYNAME)
    );
}
