use open62541::{DataType, ServerBuilder, ua};

#[test]
fn builtin_scalars_extract_as_masks() {
    fn check<T: DataType + PartialEq>(value: impl DataType, expected: T) {
        let variant = ua::Variant::scalar(value);

        assert_eq!(variant.as_scalar::<T>(), Some(&expected));
        assert_eq!(variant.into_scalar::<T>(), Some(expected));
    }

    check(ua::Byte::new(1), ua::AccessLevelType::CURRENTREAD);
    check(ua::Byte::new(1), ua::EventNotifierType::SUBSCRIBETOEVENTS);
    check(ua::UInt32::new(1), ua::AccessLevelExType::CURRENTREAD);
    check(
        ua::UInt16::new(1),
        ua::AccessRestrictionType::SIGNINGREQUIRED,
    );
    check(ua::UInt32::new(1), ua::AttributeWriteMask::ACCESSLEVEL);
    check(ua::UInt32::new(1), ua::NodeClassMask::OBJECT);
}

#[test]
fn const_flag_updates_preserve_unknown_bits_until_cleared() {
    // SAFETY: All u32 patterns are valid mask storage.
    const UNKNOWN: ua::TrustListMasks =
        unsafe { ua::TrustListMasks::from_u32_unchecked(0x8000_0000) };
    const SET: ua::TrustListMasks = UNKNOWN.set(&ua::TrustListMasks::ALL);
    const UNSET: ua::TrustListMasks = SET.unset(&ua::TrustListMasks::ALL);
    const CLEARED: ua::TrustListMasks = SET.clear();
    assert!(SET.all());
    assert_eq!(UNSET, UNKNOWN);
    assert_eq!(SET.with_all(false), UNKNOWN);
    assert_eq!(CLEARED, ua::TrustListMasks::NONE);
    assert!(ua::BrowseResultMask::REFERENCETYPEID.reference_type_id());
    assert!(ua::NodeClassMask::REFERENCETYPE.reference_type());
}

#[test]
fn mask_operators_delegate_to_const_operations() {
    const READ: ua::AccessLevelType = ua::AccessLevelType::CURRENTREAD;
    const WRITE: ua::AccessLevelType = ua::AccessLevelType::CURRENTWRITE;
    const BOTH: ua::AccessLevelType = READ.or(&WRITE);
    // SAFETY: Every bit pattern is valid for this integer-backed mask.
    const UNKNOWN: ua::AccessLevelType = unsafe { ua::AccessLevelType::from_u8_unchecked(0x80) };
    assert_eq!(READ | WRITE, BOTH);
    assert_eq!(BOTH & READ, READ);
    assert_eq!(!READ, ua::AccessLevelType::not(&READ));

    // SAFETY: Reading the mask's raw bit pattern has no extra preconditions.
    assert_eq!(unsafe { UNKNOWN.as_u8_unchecked() }, 0x80);
}

#[test]
fn builtin_attributes_extract_as_masks() {
    let (server, _runner) = ServerBuilder::default().build();
    let variable =
        ua::NodeId::ns0(open62541_sys::UA_NS0ID_SERVER_SERVERSTATUS_BUILDINFO_PRODUCTNAME);
    let object = ua::NodeId::ns0(open62541_sys::UA_NS0ID_SERVER);

    macro_rules! check {
        ($node:expr, $attribute:ident) => {{
            let value = server.read_attribute($node, ua::AttributeId::$attribute);

            assert!(value.scalar_value().is_some(), stringify!($attribute));
            assert!(value.into_scalar_value().is_some(), stringify!($attribute));
        }};
    }

    check!(&variable, ACCESSLEVEL_T);
    check!(&variable, USERACCESSLEVEL_T);
    check!(&variable, ACCESSLEVELEX_T);
    check!(&object, EVENTNOTIFIER_T);
    check!(&object, WRITEMASK_T);
    check!(&object, USERWRITEMASK_T);
}

#[test]
fn mask_predicates_keep_borrowed_receiver_signatures() {
    let predicate: fn(&ua::AccessLevelType) -> bool = ua::AccessLevelType::current_read;
    let empty: fn(&ua::AccessLevelType) -> bool = ua::AccessLevelType::none;
    assert!(predicate(&ua::AccessLevelType::CURRENTREAD));
    assert!(empty(&ua::AccessLevelType::NONE));
}
