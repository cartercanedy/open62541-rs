use std::fmt::Debug;
use std::ptr;

use open62541::{DataType, ValueType, ua};
use open62541_sys::{UA_decodeBinary, UA_encodeBinary};

fn binary_encode(value: &ua::Variant) -> ua::ByteString {
    let mut encoded = ua::ByteString::init();
    // SAFETY: The source and initialized output have the supplied native descriptors.
    unsafe {
        assert_eq!(
            UA_encodeBinary(
                value.as_ptr().cast(),
                ua::Variant::data_type(),
                encoded.as_mut_ptr(),
                ptr::null_mut(),
            ),
            0
        );
    }
    encoded
}

fn binary_round_trip(value: &ua::Variant) -> ua::Variant {
    let encoded = binary_encode(value);
    let mut decoded = ua::Variant::init();
    // SAFETY: The input is encoded Variant data and the initialized output owns decoded storage.
    unsafe {
        assert_eq!(
            UA_decodeBinary(
                encoded.as_ptr(),
                decoded.as_mut_ptr().cast(),
                ua::Variant::data_type(),
                ptr::null_mut(),
            ),
            0
        );
    }
    decoded
}

#[test]
fn native_mask_variants_use_enum_identity_and_int32_wire_values() {
    fn check<T: DataType<Inner = u32> + Copy + PartialEq>(index: u32) {
        // SAFETY: The generated index belongs to the native static descriptor array.
        let expected = unsafe { (&raw const open62541_sys::UA_TYPES).as_ref().unwrap() }
            .get(usize::try_from(index).unwrap())
            .expect("native mask descriptor exists");
        assert!(ptr::eq(T::data_type(), expected));
        assert_ne!(T::data_type(), ua::UInt32::data_type());

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
        assert_eq!(
            binary_encode(&native).as_bytes(),
            Some([0x06, 0xff, 0xff, 0xff, 0xff].as_slice())
        );
        let decoded = binary_round_trip(&native);
        assert_eq!(decoded.value_type(), Some(ValueType::Int32));
        assert_eq!(decoded.as_scalar::<T>(), Some(&value));
        assert_eq!(decoded.into_scalar::<T>(), Some(value));
        assert!(
            ua::Variant::scalar(ua::UInt32::new(1))
                .to_scalar::<T>()
                .is_none()
        );

        let array = ua::Variant::array([value, value].into());
        assert_eq!(array.value_type(), Some(ValueType::Enumeration));
        assert_eq!(
            binary_encode(&array).as_bytes(),
            Some(
                [
                    0x86, 2, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
                ]
                .as_slice()
            )
        );
        assert_eq!(
            binary_round_trip(&array)
                .to_array::<T>()
                .unwrap()
                .as_slice(),
            [value, value]
        );
    }

    check::<ua::TrustListMasks>(open62541_sys::UA_TYPES_TRUSTLISTMASKS);
    check::<ua::BrowseResultMask>(open62541_sys::UA_TYPES_BROWSERESULTMASK);
    check::<ua::SpecifiedAttributes>(open62541_sys::UA_TYPES_NODEATTRIBUTESMASK);
}

fn check_enum<T: DataType + PartialEq + Debug>(value: &T) {
    let native = ua::Variant::scalar(value.clone());
    let decoded = binary_round_trip(&native);
    assert_eq!(
        decoded.type_id(),
        ua::Variant::scalar(ua::Int32::new(0)).type_id()
    );
    assert_eq!(decoded.as_scalar::<T>(), Some(value));
    assert_eq!(decoded.to_scalar::<T>(), Some(value.clone()));
    assert_eq!(decoded.try_into_scalar::<T>().unwrap(), *value);

    let array = ua::Variant::array(ua::Array::from_slice(&[value.clone(), value.clone()]));
    let decoded = binary_round_trip(&array);
    let items = decoded.to_array::<T>().unwrap();
    assert_eq!(items[0], *value);
    assert_eq!(items[1], *value);
}

#[test]
fn real_binary_enum_scalars_and_arrays() {
    check_enum(&ua::NodeClass::OBJECT);
    check_enum(&ua::ApplicationType::SERVER);
    check_enum(&ua::DataChangeTrigger::STATUS);
    check_enum(&ua::FilterOperator::EQUALS);
    check_enum(&ua::MessageSecurityMode::NONE);
    check_enum(&ua::UserTokenType::ANONYMOUS);
}

#[test]
fn real_binary_allocating_alias() {
    let bytes = ua::ByteString::new(&[1, 2, 3]);
    // SAFETY: ContinuationPoint and ByteString share the same native C representation.
    let expected = unsafe { ua::ContinuationPoint::from_raw(bytes.into_raw()) };
    let native = ua::Variant::scalar(expected.clone());
    let decoded = binary_round_trip(&native);
    assert_eq!(
        decoded.as_scalar::<ua::ContinuationPoint>(),
        Some(&expected)
    );
    assert_eq!(
        decoded.try_into_scalar::<ua::ContinuationPoint>().unwrap(),
        expected
    );
}

fn json_round_trip(value: &ua::Variant) -> ua::Variant {
    let mut encoded = ua::ByteString::init();
    let mut decoded = ua::Variant::init();
    let options = open62541_sys::UA_EncodeJsonOptions {
        useReversible: true,
        ..Default::default()
    };
    // SAFETY: The input and outputs are live wrappers for the specified Variant descriptor.
    unsafe {
        assert_eq!(
            open62541_sys::UA_encodeJson(
                value.as_ptr().cast(),
                ua::Variant::data_type(),
                encoded.as_mut_ptr(),
                &raw const options
            ),
            0
        );
        assert_eq!(
            open62541_sys::UA_decodeJson(
                encoded.as_ptr(),
                decoded.as_mut_ptr().cast(),
                ua::Variant::data_type(),
                ptr::null()
            ),
            0
        );
    }
    decoded
}

fn check_json_enum<T: DataType + PartialEq + Debug>(value: &T) {
    let scalar = json_round_trip(&ua::Variant::scalar(value.clone()));
    assert_eq!(scalar.try_into_scalar::<T>().unwrap(), *value);
    let array = json_round_trip(&ua::Variant::array(ua::Array::from_slice(
        std::slice::from_ref(value),
    )));
    assert_eq!(array.to_array::<T>().unwrap()[0], *value);
}

#[test]
fn reversible_json_recovers_named_enums() {
    check_json_enum(&ua::NodeClass::OBJECT);
    check_json_enum(&ua::ApplicationType::SERVER);
    check_json_enum(&ua::DataChangeTrigger::STATUS);
    check_json_enum(&ua::FilterOperator::EQUALS);
    check_json_enum(&ua::MessageSecurityMode::NONE);
    check_json_enum(&ua::UserTokenType::ANONYMOUS);
}

fn check_independent_json_enum<T: DataType + PartialEq + Debug>(expected: &T, number: u32) {
    let native = ua::Variant::scalar(expected.clone());
    let (namespace, type_id) = native.type_id().unwrap().as_numeric().unwrap();
    assert_eq!(namespace, 0);
    let json = format!(r#"{{"UaType":22,"Value":{{"UaTypeId":"i={type_id}","UaBody":{number}}}}}"#);
    let input = ua::ByteString::new(json.as_bytes());
    let mut decoded = ua::Variant::init();
    // SAFETY: Input is a live byte string and output is an initialized Variant.
    unsafe {
        assert_eq!(
            open62541_sys::UA_decodeJson(
                input.as_ptr(),
                decoded.as_mut_ptr().cast(),
                ua::Variant::data_type(),
                ptr::null()
            ),
            0
        );
    }
    assert_eq!(decoded.try_into_scalar::<T>().unwrap(), *expected);
}

#[test]
fn independent_reversible_json_named_enums() {
    check_independent_json_enum(&ua::ApplicationType::SERVER, 0);
    check_independent_json_enum(&ua::DataChangeTrigger::STATUS, 0);
    check_independent_json_enum(&ua::FilterOperator::EQUALS, 0);
    check_independent_json_enum(&ua::MessageSecurityMode::NONE, 1);
    check_independent_json_enum(&ua::UserTokenType::ANONYMOUS, 0);
}

#[test]
fn xml_allocating_alias_round_trip() {
    let bytes = ua::ByteString::new(&[1, 2, 3]);
    // SAFETY: The C types share the same representation and this transfers ownership.
    let expected = unsafe { ua::ContinuationPoint::from_raw(bytes.into_raw()) };
    let native = ua::Variant::scalar(expected.clone());
    let mut encoded = ua::ByteString::init();
    let mut decoded = ua::Variant::init();
    // SAFETY: The input and outputs are live wrappers for the specified Variant descriptor.
    unsafe {
        assert_eq!(
            open62541_sys::UA_encodeXml(
                native.as_ptr().cast(),
                ua::Variant::data_type(),
                encoded.as_mut_ptr(),
                ptr::null()
            ),
            0
        );
        assert_eq!(
            open62541_sys::UA_decodeXml(
                encoded.as_ptr(),
                decoded.as_mut_ptr().cast(),
                ua::Variant::data_type(),
                ptr::null()
            ),
            0
        );
    }
    assert_eq!(
        decoded.try_into_scalar::<ua::ContinuationPoint>().unwrap(),
        expected
    );
}

#[test]
fn binary_preserves_allocating_struct_identity() {
    let raw = open62541_sys::UA_ReferenceDescription {
        browseName: ua::QualifiedName::new(2, "sample").into_raw(),
        ..Default::default()
    };
    // SAFETY: The structure owns the transferred QualifiedName; other fields are zero initialized.
    let expected = unsafe { ua::ReferenceDescription::from_raw(raw) };
    let decoded = binary_round_trip(&ua::Variant::scalar(expected.clone()));
    assert!(decoded.as_scalar::<ua::QualifiedName>().is_none());
    assert_eq!(
        decoded
            .try_into_scalar::<ua::ReferenceDescription>()
            .unwrap(),
        expected
    );
}

#[test]
fn binary_and_json_alias_arrays() {
    let bytes = ua::ByteString::new(&[4, 5]);
    // SAFETY: ContinuationPoint has the same native C representation as ByteString.
    let expected = unsafe { ua::ContinuationPoint::from_raw(bytes.into_raw()) };
    let source = ua::Variant::array(ua::Array::from_slice(std::slice::from_ref(&expected)));
    for decoded in [binary_round_trip(&source), json_round_trip(&source)] {
        assert_eq!(
            decoded.to_array::<ua::ContinuationPoint>().unwrap()[0],
            expected
        );
    }
}

#[test]
fn decoded_extension_object_preserves_concrete_identity() {
    let raw = open62541_sys::UA_ReferenceDescription {
        browseName: ua::QualifiedName::new(2, "sample").into_raw(),
        ..Default::default()
    };
    // SAFETY: The structure owns its transferred QualifiedName and initialized other fields.
    let expected = unsafe { ua::ReferenceDescription::from_raw(raw) };
    let object = ua::ExtensionObject::new(&expected);
    assert_eq!(
        object.decoded_content::<ua::ReferenceDescription>(),
        Some(&expected)
    );
    assert!(object.decoded_content::<ua::QualifiedName>().is_none());

    let decoded = binary_round_trip(&ua::Variant::scalar(object));
    assert_eq!(
        decoded
            .try_into_scalar::<ua::ReferenceDescription>()
            .unwrap(),
        expected
    );
}

#[test]
fn unknown_extension_object_stays_encoded() {
    let type_id = ua::NodeId::numeric(1, 1234);
    let bytes = ua::ByteString::new(&[7, 8, 9]);
    let mut raw = open62541_sys::UA_ExtensionObject {
        encoding: open62541_sys::UA_ExtensionObjectEncoding::UA_EXTENSIONOBJECT_ENCODED_BYTESTRING,
        ..Default::default()
    };
    // SAFETY: The tag selects this union arm, and ownership transfers to the extension object.
    unsafe {
        let content = raw.content.encoded.as_mut();
        content.typeId = type_id.clone().into_raw();
        content.body = bytes.clone().into_raw();
    }
    // SAFETY: The tag and union contents describe a valid owned encoded object.
    let object = unsafe { ua::ExtensionObject::from_raw(raw) };
    let decoded = binary_round_trip(&ua::Variant::scalar(object));
    let object = decoded.try_into_scalar::<ua::ExtensionObject>().unwrap();
    let (decoded_id, decoded_bytes) = object.encoded_content_bytestring().unwrap();
    assert_eq!(decoded_id, &type_id);
    assert_eq!(decoded_bytes, &bytes);
    assert!(
        object
            .decoded_content::<ua::ReferenceDescription>()
            .is_none()
    );
}

#[test]
fn enum_matrix_extraction_keeps_source_dimensions() {
    let native = ua::Variant::array(ua::Array::from_slice(&[
        ua::NodeClass::OBJECT,
        ua::NodeClass::VARIABLE,
        ua::NodeClass::OBJECT,
        ua::NodeClass::VARIABLE,
    ]));
    let mut raw = native.into_raw();
    // SAFETY: The Variant takes ownership of a C-allocated UInt32 dimensions array.
    unsafe {
        raw.arrayDimensions = open62541_sys::UA_Array_new(2, ua::UInt32::data_type()).cast();
        assert!(!raw.arrayDimensions.is_null());
        raw.arrayDimensions.write(2);
        raw.arrayDimensions.add(1).write(2);
        raw.arrayDimensionsSize = 2;
    }
    // SAFETY: The four elements and the owned 2x2 dimensions array form a valid matrix.
    let native = unsafe { ua::Variant::from_raw(raw) };

    for decoded in [binary_round_trip(&native), json_round_trip(&native)] {
        assert!(decoded.as_scalar::<ua::NodeClass>().is_none());
        let values = decoded.to_array::<ua::NodeClass>().unwrap();
        assert_eq!(values.len(), 4);
        assert_eq!(values[1], ua::NodeClass::VARIABLE);
        // SAFETY: The decoder produced a valid Variant; we only inspect its dimensions.
        unsafe {
            let raw = &*decoded.as_ptr();
            assert_eq!(raw.arrayDimensionsSize, 2);
            assert_eq!(std::slice::from_raw_parts(raw.arrayDimensions, 2), &[2, 2]);
        }
    }
}
