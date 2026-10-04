use std::{mem, ptr, sync::OnceLock};

use open62541::{DataType, ua};
use open62541_sys::{
    UA_DataType, UA_DataTypeArray, UA_DataTypeKind, UA_DataTypeMember, UA_DecodeBinaryOptions,
    UA_String, UA_decodeBinary, UA_encodeBinary,
};

#[repr(C)]
struct CustomRaw {
    text: UA_String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct Custom(ua::String);

struct Metadata {
    descriptor: UA_DataType,
    _member: Box<UA_DataTypeMember>,
}

// SAFETY: The pointers refer to immutable metadata owned for the lifetime of the program.
unsafe impl Send for Metadata {}
// SAFETY: Neither Rust nor C mutates this metadata after initialization.
unsafe impl Sync for Metadata {}

// SAFETY: Custom and CustomRaw each transparently contain the same owned C String storage.
// The structure descriptor describes one String at offset zero, with its matching lifecycle.
unsafe impl DataType for Custom {
    type Inner = CustomRaw;

    fn data_type() -> *const UA_DataType {
        static METADATA: OnceLock<Metadata> = OnceLock::new();
        &raw const METADATA
            .get_or_init(|| {
                assert_eq!(mem::size_of::<Self>(), mem::size_of::<CustomRaw>());
                assert_eq!(mem::align_of::<Self>(), mem::align_of::<CustomRaw>());
                let mut member = Box::new(UA_DataTypeMember {
                    memberName: c"Text".as_ptr(),
                    memberType: ua::String::data_type(),
                    ..Default::default()
                });
                let mut descriptor = UA_DataType {
                    typeName: c"Custom".as_ptr(),
                    typeId: ua::NodeId::numeric(1, 5001).into_raw(),
                    binaryEncodingId: ua::NodeId::numeric(1, 5002).into_raw(),
                    members: &raw mut *member,
                    ..Default::default()
                };
                descriptor.set_memSize(u32::try_from(mem::size_of::<CustomRaw>()).unwrap());
                descriptor.set_typeKind(u32::from_ne_bytes(
                    UA_DataTypeKind::UA_DATATYPEKIND_STRUCTURE.0.to_ne_bytes(),
                ));
                descriptor.set_membersSize(1);
                Metadata {
                    descriptor,
                    _member: member,
                }
            })
            .descriptor
    }

    unsafe fn from_raw(value: Self::Inner) -> Self {
        // SAFETY: The caller transfers the valid owned String field.
        Self(unsafe { ua::String::from_raw(value.text) })
    }

    fn into_raw(self) -> Self::Inner {
        CustomRaw {
            text: self.0.into_raw(),
        }
    }
}

#[test]
fn custom_structure_requires_decoder_registration() {
    let expected = Custom(ua::String::new("registered payload").unwrap());
    let native = ua::Variant::scalar(expected.clone());
    let mut encoded = ua::ByteString::init();
    // SAFETY: Both values are initialized wrappers with the specified native descriptors.
    unsafe {
        assert_eq!(
            UA_encodeBinary(
                native.as_ptr().cast(),
                ua::Variant::data_type(),
                encoded.as_mut_ptr(),
                ptr::null_mut(),
            ),
            0
        );
    }

    let registration = UA_DataTypeArray {
        typesSize: 1,
        types: Custom::data_type().cast_mut(),
        ..Default::default()
    };
    let mut options = UA_DecodeBinaryOptions {
        customTypes: &raw const registration,
        ..Default::default()
    };
    let mut registered = ua::Variant::init();
    let mut unregistered = ua::Variant::init();
    // SAFETY: The descriptors remain live during decoding and the outputs own all decoded data.
    unsafe {
        assert_eq!(
            UA_decodeBinary(
                encoded.as_ptr(),
                registered.as_mut_ptr().cast(),
                ua::Variant::data_type(),
                &raw mut options,
            ),
            0
        );
        assert_eq!(
            UA_decodeBinary(
                encoded.as_ptr(),
                unregistered.as_mut_ptr().cast(),
                ua::Variant::data_type(),
                ptr::null_mut(),
            ),
            0
        );
    }
    assert!(registered.as_scalar::<ua::String>().is_none());
    assert_eq!(registered.try_into_scalar::<Custom>().unwrap(), expected);
    assert!(unregistered.as_scalar::<Custom>().is_none());
    let object = unregistered
        .try_into_scalar::<ua::ExtensionObject>()
        .unwrap();
    assert_eq!(
        object.encoded_type_id(),
        Some(&ua::NodeId::numeric(1, 5002))
    );
    assert!(object.encoded_content_bytestring().is_some());
}
