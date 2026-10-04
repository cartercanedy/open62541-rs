//! Native datatype contracts and crate-private declaration macros.
//!
//! Choose a declaration by storage and representation policy:
//!
//! - [`ua_primitive!`]: scalar C storage with [`Clone`] and [`Copy`]. Tuple declarations also generate
//!   const `new` and `value` methods; bare declarations leave specialized methods handwritten.
//! - [`ua_enum!`]: open bindgen enums with named/[`ua::Int32`] extraction. Helper enums explicitly select
//!   primitive Copy storage and its descriptor with `ua_inner = UInt32`.
//! - `ua_optionset!`: integer-backed flags with explicit `Byte`, `UInt16`, or `UInt32` storage/descriptors.
//! - [`ua_alias!`]: builtin-backed aliases. [`ua::String`] and [`ua::ByteString`] aliases own allocations and accept
//!   both named and builtin descriptors. Scalar aliases use their selected builtin storage and
//!   receive [`Clone`] and [`Copy`] automatically.
//! - [`ua_struct!`]: composite C storage with deep clone and cleanup. A fieldless declaration leaves
//!   specialized access handwritten.
//!
//! In [`ua_struct!`], every array member is declared as `[T]`. The generated accessors borrow an
//! immutable slice, borrow a mutable slice through `<field>_mut`, or transfer ownership through
//! `take_<field>`. Callers choose explicitly when to copy elements into an owned container.
//! Constant assertions check array length/pointer layout, and generated tests check declarations
//! against C member metadata. Neither adds work to the accessors.
//!
//! Every declared field also generates an owned `with_<field>` builder. Wrapper fields transfer
//! their contents, array fields transfer a native [`ua::Array<T>`], and scalar fields use const setters.
//! `#[builder(skip)]` opts out when assigning a field needs coupled state changes, such as a
//! presence flag or a union payload. Field documentation is attached to the generated builder.
//! Callers validate and construct wrapper values before passing them to a builder.
//!
//! Reference fields borrow matching native wrappers, including enums. `#[from_inner]` copies a
//! Copy wrapper. `#[enum_copy]` copies a [`ua::UInt32`] helper from
//! a bindgen enum field. `#[skip(uses = Self::method)]` records a field whose accessor needs
//! handwritten handling. Optional pointers and discriminated storage must not be
//! treated as ordinary inline fields; keep their checks in specialized accessors.
//!
//! Declarations share native trait implementations and lifecycle helpers. Representation-compatible
//! extraction belongs to [`DataType`]; semantic adapters can instead implement [`DataTypeExt`](super::DataTypeExt) through
//! a native intermediary. The native blanket implementation uses `Self` as that intermediary.

use std::{
    ffi::{CStr, c_void},
    fmt::Debug,
    mem::{self, MaybeUninit},
    ptr,
};

use open62541_sys::{
    UA_DataType, UA_Order, UA_STATUSCODE_GOOD, UA_clear, UA_copy, UA_init, UA_new, UA_order,
    UA_print,
};

use crate::ua;

/// Transparent wrapper for OPC UA data type.
///
/// # Safety
///
/// It must be possible to transmute between the type that implements [`DataType`] and the inner
/// type `Inner`. This implies that `#[repr(transparent)]` must be used on types that implement
/// this trait and the inner type must be `Inner`.
///
/// The returned datatype descriptor must be valid for the lifetime of the program and describe
/// `Inner`'s size, alignment, initialization, copying, and cleanup. Values produced by those C
/// operations must be valid values of the Rust wrapper. This contract applies even when an
/// extraction hook is overridden.
///
/// In addition, the inner type must not contain self-references (in Rust terms, it would have to
/// implement the [`Unpin`] trait). This is usually the case for types from [`open62541_sys`], as
/// they are regularly passed by value to functions in order to transfer ownership.
///
/// [`Inner`]: DataType::Inner
pub unsafe trait DataType: Debug + Clone {
    /// Inner type.
    ///
    /// It must be possible to transmute between the inner type and the type that implements
    /// [`DataType`].
    type Inner;

    /// Gets `open62541` data type record.
    ///
    /// The result can be passed to functions in `open62541` that deal with arbitrary data types.
    #[must_use]
    fn data_type() -> *const UA_DataType;

    /// Returns whether a variant descriptor can be viewed as this type.
    ///
    /// Implementors may accept an additional descriptor only when its values have the same
    /// layout, alignment, valid bit patterns, and ownership requirements as `Self::Inner`.
    /// These requirements are part of the safety contract of the [`DataType`] implementation.
    #[must_use]
    fn accepts_variant_type(data_type: *const UA_DataType) -> bool {
        data_type == Self::data_type()
    }

    /// Extracts an owned value from a variant. Implementations may provide additional conversions.
    ///
    /// # Errors
    ///
    /// Returns an error when the variant cannot be converted to this type.
    fn try_from_variant(value: ua::Variant) -> crate::Result<Self> {
        value.try_into_scalar_matching::<Self>()
    }

    /// Constructs an owned value from a borrowed variant.
    ///
    /// The default clones a supported scalar after checking its representation. Override this
    /// method for conversions that construct a value without permitting a borrowed `&Self` view.
    /// This method does not clone the input to call [`Self::try_from_variant()`]. Custom copying
    /// and consuming conversions require separate overrides, which may explicitly delegate to one
    /// another. Extracting [`Variant`] itself clones its contents as the requested result.
    ///
    /// # Errors
    ///
    /// Returns an error when the variant cannot be converted to this type.
    ///
    /// [`Variant`]: ua::Variant
    fn try_clone_from_variant(value: &ua::Variant) -> crate::Result<Self> {
        Self::borrow_from_variant(value)
            .cloned()
            .ok_or_else(|| crate::Error::new(ua::StatusCode::BADTYPEMISMATCH))
    }

    /// Borrows a value when its in-memory representation can safely be viewed as this type.
    #[must_use]
    fn borrow_from_variant(value: &ua::Variant) -> Option<&Self> {
        value.as_scalar_matching::<Self>()
    }

    /// Copies an array of values when its representation is supported.
    #[must_use]
    fn array_from_variant(value: &ua::Variant) -> Option<ua::Array<Self>> {
        value.to_array_matching::<Self>()
    }

    /// Gets data type name.
    #[must_use]
    fn type_name() -> &'static str {
        let data_type = Self::data_type();
        // SAFETY: `data_type` is a valid pointer.
        unsafe { CStr::from_ptr((*data_type).typeName) }
            .to_str()
            // PANIC: `typeName` is an ASCII string.
            .expect("string should be valid")
    }

    /// Creates wrapper by taking ownership of value.
    ///
    /// When `Self` is dropped, [`UA_clear()`] is used to free allocations held by the inner type.
    /// Move only values into `Self` that can be cleared in-place such as stack-allocated values
    /// (but no heap-allocated values created by [`UA_new()`]).
    ///
    /// # Safety
    ///
    /// Ownership of the value passes to `Self`. This must only be used for values that are not
    /// contained within other values that may be dropped (such as attributes in other data types).
    /// In this case use [`clone_raw()`] or [`take_raw()`] instead to clone or move data instead of
    /// taking ownership.
    ///
    /// [`UA_new()`]: open62541_sys::UA_new
    /// [`clone_raw()`]: DataType::clone_raw
    /// [`take_raw()`]: DataType::take_raw
    #[must_use]
    unsafe fn from_raw(src: Self::Inner) -> Self;

    /// Gives up ownership and returns value.
    ///
    /// The returned value must be re-wrapped with [`from_raw()`] or cleared manually with
    /// [`UA_clear()`] to free internal allocations and not leak memory.
    ///
    /// [`from_raw()`]: DataType::from_raw
    #[must_use]
    fn into_raw(self) -> Self::Inner;

    /// Creates wrapper initialized with defaults.
    ///
    /// This uses [`UA_init()`] to initialize the value and make all attributes well-defined.
    /// Depending on the type, additional attributes may need to be initialized for the value to be
    /// actually useful afterwards.
    #[must_use]
    fn init() -> Self {
        let mut inner = MaybeUninit::<Self::Inner>::uninit();
        // `UA_init()` may depend on the specific data type. The current implementation just
        // zero-initializes the memory region. We could use `MaybeUninit::zeroed()` instead but we
        // want to allow `open62541` to use a different implementation in the future if necessary.
        unsafe {
            UA_init(inner.as_mut_ptr().cast::<c_void>(), Self::data_type());
        }
        // SAFETY: We just made sure that the memory region is initialized.
        let inner = unsafe { inner.assume_init() };
        // SAFETY: We pass a value without pointers to it into `Self`.
        unsafe { Self::from_raw(inner) }
    }

    /// Creates wrapper reference from value.
    #[must_use]
    fn raw_ref(src: &Self::Inner) -> &Self {
        data_type_ref(src)
    }

    /// Creates mutable wrapper reference from value.
    #[must_use]
    fn raw_mut(src: &mut Self::Inner) -> &mut Self {
        data_type_ref_mut(src)
    }

    /// Creates wrapper by cloning value from `src`.
    ///
    /// This uses [`UA_copy()`] to deeply copy an existing value without transferring ownership.
    ///
    /// The original value must be cleared with [`UA_clear()`], or deleted with [`UA_delete()`] if
    /// allocated on the heap, to avoid memory leaks. If `src` is borrowed from another data type
    /// wrapper, that wrapper will make sure of this.
    ///
    /// [`UA_delete()`]: open62541_sys::UA_delete
    #[must_use]
    fn clone_raw(src: &Self::Inner) -> Self {
        let src: *const Self::Inner = src;
        // `UA_copy()` does not clean up the target value before copying into it, so we may use an
        // uninitialized memory region here.
        let mut dst = MaybeUninit::<Self::Inner>::uninit();
        let result = unsafe {
            UA_copy(
                src.cast::<c_void>(),
                dst.as_mut_ptr().cast::<c_void>(),
                Self::data_type(),
            )
        };
        assert_eq!(result, UA_STATUSCODE_GOOD, "should have copied value");
        // SAFETY: We just made sure that the memory region is initialized.
        let dst = unsafe { dst.assume_init() };
        // SAFETY: We pass a value without pointers to it into `Self`.
        unsafe { Self::from_raw(dst) }
    }

    /// Creates wrapper by moving value from `src`.
    ///
    /// This moves an existing value and uses [`UA_init()`] to leave an initialized value behind.
    #[must_use]
    fn take_raw(src: &mut Self::Inner) -> Self {
        // Take out source value and leave behind a freshly initialized value that gets cleaned up
        // when the external owner frees memory.
        let src = mem::replace(src, Self::init().into_raw());
        // SAFETY: We have replaced the original source with a freshly initialized value. There is
        // no other owner and we take ownership.
        unsafe { Self::from_raw(src) }
    }

    /// Clones value into `dst`.
    ///
    /// This uses [`UA_copy()`] to deeply copy an existing value without transferring ownership.
    ///
    /// Existing data in `dst` is cleared with [`UA_clear()`] before cloning the value; it is safe
    /// to use this operation on already initialized target values.
    fn clone_into_raw(&self, dst: &mut Self::Inner) {
        let dst: *mut Self::Inner = dst;
        // `UA_copy()` does not clean up the target value before copying into it, so we use
        // `UA_clear()` first to free dynamically allocated memory held by the current value.
        unsafe {
            UA_clear(dst.cast::<c_void>(), Self::data_type());
        }
        // Copy ourselves into the target. This duplicates and allocates memory if necessary to
        // store a copy of the inner value.
        let result = unsafe {
            UA_copy(
                self.as_ptr().cast::<c_void>(),
                dst.cast::<c_void>(),
                Self::data_type(),
            )
        };
        assert_eq!(result, UA_STATUSCODE_GOOD, "should have copied value");
    }

    /// Moves value into `dst`, giving up ownership.
    ///
    /// Existing data in `dst` is cleared with [`UA_clear()`] before moving the value; it is safe
    /// to use this operation on already initialized target values.
    ///
    /// After this, it is the responsibility of `dst` to eventually clean up the data.
    fn move_into_raw(self, dst: &mut Self::Inner) {
        let dst: *mut Self::Inner = dst;
        // Use `UA_clear()` first to free dynamically allocated memory held by the current value.
        unsafe {
            UA_clear(dst.cast::<c_void>(), Self::data_type());
        }
        // Move ourselves into the target. This keeps existing memory allocations but we do not
        // reference them anymore because `into_raw()` gives up ownership.
        unsafe { *dst = self.into_raw() };
    }

    /// Leaks wrapped value onto the heap.
    ///
    /// This turns a stack-allocated value into a heap-allocated one, without issuing a deep copy.
    /// In other words, only the local memory allocation is copied (moved from stack to heap) and
    /// any memory that is already heap-allocated (e.g. string contents) stays where it is.
    ///
    /// The returned value must be passed into another owned data structure or freed manually with
    /// [`UA_delete()`] to free internal allocations and not leak memory.
    ///
    /// [`UA_delete()`]: open62541_sys::UA_delete
    #[must_use]
    fn leak_into_raw(self) -> *mut Self::Inner {
        // Use `UA_new()` to create heap allocation that can be cleaned up with `UA_free()`.
        let dst = unsafe { UA_new(Self::data_type()) }.cast::<Self::Inner>();
        // Check that heap allocation was successful (we might be out of memory).
        assert!(!dst.is_null(), "should have allocated heap memory");
        // SAFETY: Pointer is valid (non-zero) because we just checked it.
        self.move_into_raw(unsafe { dst.as_mut().unwrap_unchecked() });
        dst
    }

    /// Creates copy without giving up ownership.
    ///
    /// # Safety
    ///
    /// Think twice before using this. Pointers to dynamically allocated attributes within `Self`
    /// are copied and will become dangling pointers when `this` is dropped. Vice versa, directly
    /// (or indirectly) freeing the copied value would double-free memory when `this` is dropped.
    ///
    /// This function is necessary because some functions in `open62541` take arguments by value
    /// instead of by pointer _without taking ownership_ and make the caller responsible for
    /// cleaning up after the call has returned.
    #[must_use]
    unsafe fn to_raw_copy(this: &Self) -> Self::Inner {
        // SAFETY: This creates a copy of the inner value despite it not being `Copy`. Extreme care
        // must be taken that any contained structures are not freed twice when `self` is dropped.
        unsafe { ptr::read(this.as_ptr()) }
    }

    /// Returns shared reference to value.
    ///
    /// # Safety
    ///
    /// The value is owned by `Self`. Ownership must not be given away, in whole or in parts. This
    /// may happen when `open62541` functions are called that take ownership of values by pointer.
    #[must_use]
    unsafe fn as_ref(&self) -> &Self::Inner {
        // SAFETY: We wrap the pointer into a reference below to ensure upholding lifetime rules.
        let ptr = unsafe { self.as_ptr() };
        // SAFETY: `DataType` guarantees that we can transmute between `Self` and the inner type.
        let ptr = unsafe { ptr.as_ref() };
        // SAFETY: Pointer is valid (non-zero) because it comes from a reference.
        unsafe { ptr.unwrap_unchecked() }
    }

    /// Returns exclusive reference to value.
    ///
    /// # Safety
    ///
    /// The value is owned by `Self`. Ownership must not be given away, in whole or in parts. This
    /// may happen when `open62541` functions are called that take ownership of values by pointer.
    #[must_use]
    unsafe fn as_mut(&mut self) -> &mut Self::Inner {
        // SAFETY: We wrap the pointer into a reference below to ensure upholding lifetime rules.
        let ptr = unsafe { self.as_mut_ptr() };
        // SAFETY: `DataType` guarantees that we can transmute between `Self` and the inner type.
        let ptr = unsafe { ptr.as_mut() };
        // SAFETY: Pointer is valid (non-zero) because it comes from a reference.
        unsafe { ptr.unwrap_unchecked() }
    }

    /// Returns const pointer to value.
    ///
    /// # Safety
    ///
    /// The value is owned by `Self`. Ownership must not be given away, in whole or in parts. This
    /// may happen when `open62541` functions are called that take ownership of values by pointer.
    #[must_use]
    unsafe fn as_ptr(&self) -> *const Self::Inner {
        let this: *const Self = self;
        // This transmutes between `Self` and the inner type through `cast()`. Types that implement
        // `DataType` guarantee that we can transmute between them and their inner type, so this is
        // okay.
        this.cast::<Self::Inner>()
    }

    /// Returns mutable pointer to value.
    ///
    /// # Safety
    ///
    /// The value is owned by `Self`. Ownership must not be given away, in whole or in parts. This
    /// may happen when `open62541` functions are called that take ownership of values by pointer.
    #[must_use]
    unsafe fn as_mut_ptr(&mut self) -> *mut Self::Inner {
        let this: *mut Self = self;
        // This transmutes between `Self` and the inner type through `cast()`. Types that implement
        // `DataType` guarantee that we can transmute between them and their inner type, so this is
        // okay.
        this.cast::<Self::Inner>()
    }

    /// Prints value to string.
    ///
    /// This uses [`UA_print()`] to generate the string representation.
    ///
    /// # Note
    ///
    /// The string representation is not guaranteed to be stable across versions.
    #[must_use]
    fn print(this: &Self) -> Option<ua::String> {
        let mut output = ua::String::init();
        let result = unsafe {
            UA_print(
                this.as_ptr().cast::<c_void>(),
                Self::data_type(),
                <ua::String as DataType>::as_mut_ptr(&mut output),
            )
        };
        (result == UA_STATUSCODE_GOOD).then_some(output)
    }

    /// Compares value to other.
    ///
    /// This uses [`UA_order()`] to derive a total ordering between values.
    #[must_use]
    fn order(this: &Self, other: &Self) -> UA_Order {
        unsafe {
            UA_order(
                this.as_ptr().cast::<std::ffi::c_void>(),
                other.as_ptr().cast::<std::ffi::c_void>(),
                Self::data_type(),
            )
        }
    }
}

pub(crate) const fn data_type_ref<T: DataType>(value: &<T as DataType>::Inner) -> &T {
    let src: *const T::Inner = value;
    // This transmutes between the inner type and `Self` through `cast()`. Types that implement
    // `DataType` guarantee that we can transmute between them and their inner type, so this is
    // okay.
    let ptr = src.cast::<T>();
    // SAFETY: `DataType` guarantees that we can transmute between `Self` and the inner type.
    let ptr = unsafe { ptr.as_ref() };
    // SAFETY: Pointer is valid (non-zero) because it comes from a reference.
    unsafe { ptr.unwrap_unchecked() }
}

pub(crate) const fn data_type_ref_mut<T: DataType>(value: &mut <T as DataType>::Inner) -> &mut T {
    let src: *mut T::Inner = value;
    // This transmutes between the inner type and `Self` through `cast()`. Types that implement
    // `DataType` guarantee that we can transmute between them and their inner type, so this is
    // okay.
    let ptr = src.cast::<T>();
    // SAFETY: `DataType` guarantees that we can transmute between `Self` and the inner type.
    let ptr = unsafe { ptr.as_mut() };
    // SAFETY: Pointer is valid (non-zero) because it comes from a reference.
    unsafe { ptr.unwrap_unchecked() }
}

// Shared assertions used by declaration-generated tests; none are part of runtime code.
#[cfg(test)]
pub(crate) fn verify_scalar_layout<T: DataType>(kind: u32, size: usize, alignment: usize) {
    // SAFETY: DataType supplies a valid static descriptor.
    let descriptor = unsafe { &*T::data_type() };
    assert_eq!(descriptor.typeKind(), kind);
    assert_eq!(descriptor.pointerFree(), 1);
    assert_eq!(usize::try_from(descriptor.memSize()).unwrap(), size);
    assert_eq!(mem::size_of::<T>(), size);
    assert_eq!(mem::align_of::<T>(), alignment);
    assert!(!mem::needs_drop::<T>());
}

#[cfg(all(test, feature = "serde"))]
pub(crate) fn verify_enum_serde<T>(make: impl Fn(u32) -> T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + Debug,
{
    let value = make(u32::MAX);
    assert_eq!(serde_json::to_string(&value).unwrap(), "-1");
    assert_eq!(serde_json::from_str::<T>("-1").unwrap(), value);
    assert!(serde_json::from_str::<T>("2147483648").is_err());
    assert!(serde_json::from_str::<T>("-2147483649").is_err());
}

#[cfg(all(test, feature = "serde"))]
pub(crate) fn verify_unsigned_serde<T>(max: u64)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + Debug,
{
    let json = max.to_string();
    let parsed = serde_json::from_str::<T>(&json).unwrap();
    assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    assert!(serde_json::from_str::<T>("-1").is_err());
    assert!(serde_json::from_str::<T>(&(max + 1).to_string()).is_err());
}

// Common native trait and comparison implementations for the declaration classes.
macro_rules! native_type_impl {
    ($name:ident, $inner:ident, $index:ident, native_eq $(; $($extra:item)*)?) => {
        $crate::data_type::native_type_impl!($name, $inner, $index $(; $($extra)*)?);
        impl std::cmp::PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                <Self as std::cmp::Ord>::cmp(self, other) == std::cmp::Ordering::Equal
            }
        }

        // The implementation of [`UA_order()`] ensures an equivalence relation. Among others, the
        // comparison of floating point numbers deviates from IEEE 754 and handles NaN as proper
        // values.
        impl std::cmp::Eq for $name {}

    };
    ($name:ident, $inner:ident, $index:ident $(; $($extra:item)*)?) => {
        // SAFETY: The types in `open62541` can be sent across thread boundaries. They contain
        // pointers but all internal dynamic allocations contain only their own data (nothing is
        // shared) and the allocations need not be deallocated in the same thread where they were
        // allocated.
        unsafe impl Send for $name {}

        // SAFETY: References to [`DataType`] may be sent across threads. The inner types would not
        // allow this (because pointers are used to pass ownership) but we must unwrap our wrapper
        // types in this case which is only implemented for owned values.
        unsafe impl Sync for $name {}

        // SAFETY: We can transmute between our wrapper type and the inner type. This is ensured by
        // using `#[repr(transparent)]` on the type definition.
        unsafe impl $crate::DataType for $name {
            type Inner = open62541_sys::$inner;

            $($($extra)*)?

            fn data_type() -> *const open62541_sys::UA_DataType {
                // PANIC: Value must fit into `usize` to allow indexing.
                let index = usize::try_from(open62541_sys::$index).unwrap();
                let ua_types = &raw const open62541_sys::UA_TYPES;
                // SAFETY: Pointer is non-zero, aligned, correct type.
                // PANIC: The given index is valid within `UA_TYPES`.
                unsafe { (*ua_types).get(index) }.unwrap()
            }

            unsafe fn from_raw(src: Self::Inner) -> Self {
                $name(src)
            }

            fn into_raw(self) -> Self::Inner {
                // SAFETY: We can transmute with `#[repr(transparent)]` on the type definition. (We
                // cannot simply unwrap due to an implementation of `Drop`.)
                unsafe { std::mem::transmute(self) }
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let output = <Self as $crate::DataType>::print(self);
                let string = output.as_ref().and_then(|output| output.as_str());
                // Do not apply any formatting flags to the stringified value.
                f.write_str(string.unwrap_or(stringify!($name)))
            }
        }

        impl std::cmp::PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(<Self as std::cmp::Ord>::cmp(self, other))
            }
        }

        // The implementation of [`UA_order()`] ensures a total order.
        impl std::cmp::Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                let result = <Self as $crate::DataType>::order(self, other);
                match result {
                    open62541_sys::UA_Order::UA_ORDER_LESS => std::cmp::Ordering::Less,
                    open62541_sys::UA_Order::UA_ORDER_EQ => std::cmp::Ordering::Equal,
                    open62541_sys::UA_Order::UA_ORDER_MORE => std::cmp::Ordering::Greater,
                    _ => panic!("should return valid order"),
                }
            }
        }
    };
}

pub(crate) use native_type_impl;

// Composite storage is deeply cloned and cleared through its native descriptor.
macro_rules! owned_lifecycle {
    ($name:ident) => {
        impl Clone for $name {
            fn clone(&self) -> Self {
                <Self as $crate::DataType>::clone_raw(&self.0)
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                // SAFETY: The wrapper owns a valid value for this descriptor.
                unsafe {
                    open62541_sys::UA_clear(
                        (&raw mut self.0).cast::<std::ffi::c_void>(),
                        <Self as $crate::DataType>::data_type(),
                    )
                }
            }
        }
    };
}
pub(crate) use owned_lifecycle;

macro_rules! ua_enum {
    (#[ua_inner = UInt32] pub enum $name:ident { $( $value:ident = $raw:expr ),* $(,)? }) => {
        #[doc = concat!("Integer-backed wrapper for `", stringify!($name), "`.")]
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $name(open62541_sys::UA_UInt32);

        $crate::data_type::native_type_impl!($name, UA_UInt32, UA_TYPES_UINT32);

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_u32(self.0)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <u32 as serde::Deserialize>::deserialize(deserializer).map(Self)
            }
        }

        paste::paste! {
            impl $name {
                /// Constructs the wrapper from its native C integer representation.
                ///
                /// # Safety
                ///
                /// This integer conversion has no additional memory-safety preconditions.
                #[must_use]
                pub const unsafe fn from_ua_unchecked(value: &open62541_sys::[<UA_ $name>]) -> Self {
                    Self(u32::from_ne_bytes(value.0.to_ne_bytes()))
                }

                /// Constructs the wrapper from the raw integer bits.
                ///
                /// # Safety
                ///
                /// Every bit pattern is valid for the primitive storage.
                #[must_use]
                pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
                    Self(value)
                }

                /// Returns the raw integer bits.
                ///
                /// # Safety
                ///
                /// Reading the bits has no additional memory-safety preconditions.
                #[must_use]
                pub const unsafe fn as_u32_unchecked(&self) -> u32 {
                    self.0
                }
            }

            const _: () = {
                assert!(
                    std::mem::size_of::<open62541_sys::[<UA_ $name>]>()
                        == std::mem::size_of::<u32>()
                );
                assert!(
                    std::mem::align_of::<open62541_sys::[<UA_ $name>]>()
                        == std::mem::align_of::<u32>()
                );
            };
        }

        $crate::data_type::ua_enum!(
            @variants $name, [$( $value = $raw ),*]
        );
        #[cfg(test)]
        paste::paste! {
            #[test]
            fn [<verify_ $name:snake _uint32_contract>]() {
                $crate::data_type::verify_scalar_layout::<$name>(
                    u32::from_ne_bytes(open62541_sys::UA_DataTypeKind::UA_DATATYPEKIND_UINT32.0.to_ne_bytes()),
                    std::mem::size_of::<u32>(), std::mem::align_of::<u32>(),
                );
                #[cfg(feature = "serde")]
                $crate::data_type::verify_unsigned_serde::<$name>(u64::from(u32::MAX));
            }
        }

    };

    (pub enum $name:ident { $( $value:ident = $raw:expr ),* $(,)? }) => {
        $crate::data_type::ua_enum!(@protocol $name { $( $value = $raw ),* };
            fn accepts_variant_type(data_type: *const open62541_sys::UA_DataType) -> bool {
                data_type == Self::data_type()
                    || data_type == <crate::ua::Int32 as crate::DataType>::data_type()
            }
        );
    };
    (@protocol $name:ident { $( $value:ident = $raw:expr ),* }; $($implementation:item)+) => {
        paste::paste! {
            #[doc = concat!("Owned wrapper for `open62541_sys::UA_", stringify!($name), "`.")]
            #[repr(transparent)]
            #[derive(Clone, PartialEq, Eq)]
            pub struct $name(open62541_sys::[<UA_ $name>]);

            $crate::data_type::native_type_impl!(
                $name,
                [<UA_ $name>],
                [<UA_TYPES_ $name:upper>];
                $($implementation)+
            );
            const _: () = {
                assert!(std::mem::size_of::<open62541_sys::[<UA_ $name>]>() == std::mem::size_of::<i32>());
                assert!(std::mem::align_of::<open62541_sys::[<UA_ $name>]>() == std::mem::align_of::<i32>());
            };
            impl $name {
                $crate::data_type::ua_enum!(@raw_bindgen [<UA_ $name>]);
            }

            #[cfg(feature = "serde")]
            impl serde::Serialize for $name {
                fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    // SAFETY: Reading native enum bits has no additional preconditions.
                    let bits = unsafe { self.as_u32_unchecked() };
                    serializer.serialize_i32(i32::from_ne_bytes(bits.to_ne_bytes()))
                }
            }

            #[cfg(feature = "serde")]
            impl<'de> serde::Deserialize<'de> for $name {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    let value = <i32 as serde::Deserialize>::deserialize(deserializer)?;
                    // SAFETY: The open native enum accepts every 32-bit pattern.
                    Ok(unsafe { Self::from_u32_unchecked(u32::from_ne_bytes(value.to_ne_bytes())) })
                }
            }

            $crate::data_type::ua_enum!(
                @variants $name, [$( $value = $raw ),*]
            );
            #[cfg(test)]
            #[test]
            fn [<verify_ $name:snake _enum_contract>]() {
                $crate::data_type::verify_scalar_layout::<$name>(
                    u32::from_ne_bytes(open62541_sys::UA_DataTypeKind::UA_DATATYPEKIND_ENUM.0.to_ne_bytes()),
                    std::mem::size_of::<i32>(), std::mem::align_of::<i32>(),
                );
                #[cfg(feature = "serde")]
                $crate::data_type::verify_enum_serde::<$name>(|bits| unsafe {
                    $name::from_u32_unchecked(bits)
                });
            }

        }
    };

    // Named constants retain their C integer bits for either storage representation.
    (@variants $name:ident, [$( $value:ident = $raw:expr ),* $(,)?]) => {
        #[cfg(test)]
        const _: () = {
            $(let _: fn($name) -> bool = |value| matches!(value, $name::$value);)*
        };
        impl $name {
            $(
                #[doc = concat!("Enum variant defined by `", stringify!($raw), "`.")]
                #[expect(clippy::allow_attributes, reason = "not required for all variants")]
                #[allow(dead_code, reason = "unused `pub`-declared constants in private modules")]
                pub const $value: Self = {
                    // SAFETY: Bindgen constants have valid integer bits for this wrapper.
                    unsafe { Self::from_ua_unchecked(&$raw) }
                };

                paste::paste! {
                    // This cast is necessary on Windows builds with inner type `i32`.
                    #[expect(clippy::allow_attributes, reason = "dynamic condition")]
                    #[allow(clippy::as_conversions, trivial_numeric_casts, reason = "bindgen i32")]
                    pub const [<$value _U32>]: u32 = ($raw).0 as u32;
                }
            )*
        }

        paste::paste! {
            impl $name {
                $(
                    #[must_use]
                    pub const fn [<is_ $value:lower>](&self) -> bool {
                        // SAFETY: Reading the integer bits has no additional preconditions.
                        unsafe { self.as_u32_unchecked() == Self::[<$value _U32>] }
                    }
                )*
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    $(
                        // SAFETY: Reading the integer bits has no additional preconditions.
                        if unsafe { self.as_u32_unchecked() } == Self::[<$value _U32>] {
                            return f.write_str(stringify!($value));
                        }
                    )*
                    // SAFETY: Reading the integer bits has no additional preconditions.
                    unsafe { self.as_u32_unchecked() }.fmt(f)
                }
            }
        }
    };

    // Bindgen enum signedness varies by target; conversions preserve the integer bits.
    (@raw_bindgen $inner:ident) => {
        /// Constructs this value from its native open62541 representation.
        ///
        /// # Safety
        /// Every bit pattern is valid for the integer-backed representation.
        #[must_use]
        pub const unsafe fn from_ua_unchecked(value: &open62541_sys::$inner) -> Self {
            Self(open62541_sys::$inner(value.0))
        }

        /// Constructs this value from its raw integer bit pattern.
        ///
        /// # Safety
        /// Every bit pattern is valid for the integer-backed representation.
        #[must_use]
        pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
            #[expect(clippy::allow_attributes, reason = "cast diagnostics depend on bindgen signedness")]
            #[allow(
                clippy::as_conversions,
                clippy::as_underscore,
                clippy::cast_possible_wrap,
                trivial_numeric_casts,
                reason = "bindgen selects signed or unsigned 32-bit storage; retain every bit"
            )]
            Self(open62541_sys::$inner(value as _))
        }

        /// Returns the raw integer bit pattern.
        ///
        /// # Safety
        /// Reading the bits has no additional memory-safety preconditions.
        #[must_use]
        pub const unsafe fn as_u32_unchecked(&self) -> u32 {
            u32::from_ne_bytes(self.0.0.to_ne_bytes())
        }
    };
}
pub(crate) use ua_enum;

// Integer-backed flags only; the structured Value/ValidBits OptionSet is a separate datatype.
macro_rules! ua_optionset {
    (#[ua_inner = Byte] pub enum $name:ident $variants:tt) => {
        $crate::data_type::ua_optionset!(@flags $name : u8, Byte; $variants);
    };
    (#[ua_inner = UInt16] pub enum $name:ident $variants:tt) => {
        $crate::data_type::ua_optionset!(@flags $name : u16, UInt16; $variants);
    };
    (#[ua_inner = UInt32] pub enum $name:ident $variants:tt) => {
        $crate::data_type::ua_optionset!(@flags $name : u32, UInt32; $variants);
    };
    (@flags $name:ident : $inner_t:ident, $ua_inner:ident; {
        NONE = $none:expr,
        $( $(#[$doc:meta])* $variant:ident = $raw:expr ),+ $(,)?
    }) => {
        #[expect(clippy::allow_attributes, reason = "constant widths vary by declaration")]
        #[allow(
            clippy::as_conversions,
            clippy::cast_lossless,
            clippy::cast_sign_loss,
            trivial_numeric_casts,
            reason = "check the original constant before narrowing"
        )]
        const _: () = {
            assert!(($none as u128) <= ($inner_t::MAX as u128));
            $(assert!(($raw as u128) <= ($inner_t::MAX as u128));)+
        };

        paste::paste! {
            #[doc = concat!("Bitmask wrapper for `", stringify!($name), "`.")]
            #[repr(transparent)]
            #[derive(Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name(open62541_sys::[<UA_ $ua_inner>]);

            $crate::data_type::native_type_impl!(
                $name,
                [<UA_ $ua_inner>],
                [<UA_TYPES_ $ua_inner:upper>]
            );
        }

        #[cfg(test)]
        paste::paste! {
            const _: () = {
                let _: fn($name) -> bool = |value| matches!(value, $name::NONE);
                $(let _: fn($name) -> bool = |value| matches!(value, $name::[<$variant:upper>] );)+
            };
            #[test]
            fn [<verify_ $name:snake _flag_contract>]() {
                $crate::data_type::verify_scalar_layout::<$name>(
                    u32::from_ne_bytes(open62541_sys::UA_DataTypeKind::[<UA_DATATYPEKIND_ $ua_inner:upper>].0.to_ne_bytes()),
                    std::mem::size_of::<$inner_t>(), std::mem::align_of::<$inner_t>(),
                );
                #[cfg(feature = "serde")]
                $crate::data_type::verify_unsigned_serde::<$name>(u64::from($inner_t::MAX));
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                paste::paste! { serializer.[<serialize_ $inner_t>](self.0) }
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <$inner_t as serde::Deserialize>::deserialize(deserializer).map(Self)
            }
        }

        paste::paste! {
            impl std::ops::BitAnd for $name {
                type Output = Self;

                fn bitand(self, rhs: Self) -> Self {
                    self.and(&rhs)
                }
            }

            impl std::ops::BitOr for $name {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self {
                    self.or(&rhs)
                }
            }

            impl std::ops::Not for $name {
                type Output = Self;

                fn not(self) -> Self {
                    Self::not(&self)
                }
            }

            #[expect(clippy::allow_attributes, reason = "some implementers may be u32")]
            impl $name {
                #[allow(unused, reason = "not used by all implementers")]
                #[must_use]
                pub(crate) const fn [<as_ $inner_t>](self) -> $inner_t {
                    self.0
                }

                #[allow(unused, reason = "not used by all implementers")]
                #[must_use]
                pub(crate) const fn [<from_ $inner_t>](inner: $inner_t) -> Self {
                    // SAFETY: All bit patterns are valid for the primitive storage.
                    unsafe { Self::[<from_ $inner_t _unchecked>](inner) }
                }

                /// Constructs a mask from its raw bit pattern.
                ///
                /// # Safety
                ///
                /// Every bit pattern is valid for the primitive storage.
                #[must_use]
                pub const unsafe fn [<from_ $inner_t _unchecked>](inner: $inner_t) -> Self {
                    Self(inner)
                }

                /// Returns the raw bit pattern.
                ///
                /// # Safety
                ///
                /// Reading the bit pattern has no additional memory-safety preconditions.
                #[must_use]
                pub const unsafe fn [<as_ $inner_t _unchecked>](self) -> $inner_t {
                    self.0
                }

                #[doc = "Indicates that no fields are set."]
                #[allow(trivial_numeric_casts, reason = "some implementers may be u32")]
                #[must_use]
                pub const fn none(&self) -> bool {
                    self.0 == Self::NONE.0
                }

                #[allow(trivial_numeric_casts, reason = "some implementers may be u32")]
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::as_conversions,
                    reason = "NONE was checked before narrowing"
                )]
                pub const NONE: Self = Self($none as $inner_t);

                #[allow(trivial_numeric_casts, reason = "some implementers may be u32")]
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::as_conversions,
                    reason = "all supported mask widths fit u32"
                )]
                pub const NONE_U32: u32 = Self::NONE.0 as u32;

                #[doc = "Determines if `self`'s bit pattern contains the exact bit pattern of `other`."]
                #[must_use]
                pub const fn contains(&self, other: &Self) -> bool {
                    self.0 & other.0 == other.0
                }

                #[doc = "Yields the bitwise-or of `self` with `rhs`."]
                #[must_use]
                pub const fn or(&self, rhs: &Self) -> Self {
                    Self(self.0 | rhs.0)
                }

                #[doc = "Yields the bitwise-and of `self` with `rhs`."]
                #[must_use]
                pub const fn and(&self, rhs: &Self) -> Self {
                    Self(self.0 & rhs.0)
                }

                #[doc = "Yields the bitwise-not of `self`."]
                #[must_use]
                pub const fn not(&self) -> Self {
                    Self(!self.0)
                }

                /// Includes the supplied flag bits.
                #[must_use]
                pub const fn set(self, flags: &Self) -> Self {
                    self.or(flags)
                }

                /// Removes the supplied flag bits while preserving all others.
                #[must_use]
                pub const fn unset(self, flags: &Self) -> Self {
                    self.and(&flags.not())
                }

                /// Removes all bits, including unknown bits outside declared composites.
                #[must_use]
                pub const fn clear(self) -> Self {
                    Self(0)
                }

                $(
                    #[allow(trivial_numeric_casts, reason = "some implementers may be u32")]
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::as_conversions,
                        reason = "the original constant was checked before narrowing"
                    )]
                    pub const [<$variant:upper>]: Self = Self($raw as $inner_t);

                    #[allow(trivial_numeric_casts, reason = "some implementers may be u32")]
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::as_conversions,
                        reason = "all supported mask widths fit u32"
                    )]
                    pub const [<$variant:upper _U32>]: u32 = Self::[<$variant:upper>].0 as u32;

                    #[must_use]
                    $(
                        #[$doc]
                    )*
                    pub const fn [<$variant:snake>](&self) -> bool {
                        self.contains(&Self::[<$variant:upper>])
                    }

                    #[must_use]
                    pub const fn [<with_ $variant:snake>](self, value: bool) -> Self {
                        if value {
                            self.set(&Self::[<$variant:upper>])
                        } else {
                            self.unset(&Self::[<$variant:upper>])
                        }
                    }
                )+
            }
        }
    };

}
pub(crate) use ua_optionset;

/// Declares a native composite wrapper and its explicitly selected field accessors.
macro_rules! ua_struct {
    (pub struct $name:ident {}) => {
        $crate::data_type::ua_struct!(pub struct $name;);
    };

    (pub struct $name:ident { $($fields:tt)+ }) => {
        $crate::data_type::ua_struct!(pub struct $name;);

        impl $name {
            $crate::data_type::ua_struct!(@fields $($fields)*);
            $crate::data_type::ua_struct!(@builders [] $($fields)*);
        }

        $crate::data_type::ua_struct!(@checks $name; $($fields)*);
    };

    (pub struct $name:ident;) => {
        paste::paste! {
            #[doc = concat!("Owned wrapper for `open62541_sys::UA_", stringify!($name), "`.")]
            #[repr(transparent)]
            pub struct $name(open62541_sys::[<UA_ $name>]);

            $crate::data_type::native_type_impl!($name, [<UA_ $name>], [<UA_TYPES_ $name:upper>], native_eq);
            $crate::data_type::owned_lifecycle!($name);
        }
    };

    (@array_check $name:ident, $raw_name:ident, $arr_t:ty) => {
        paste::paste! {
            const _: fn(&<$name as $crate::DataType>::Inner) -> *mut <$arr_t as $crate::DataType>::Inner =
                |raw| raw.$raw_name;
            const _: () = {
                assert!(
                    std::mem::offset_of!(<$name as $crate::DataType>::Inner, $raw_name)
                        == std::mem::offset_of!(
                            <$name as $crate::DataType>::Inner,
                            [<$raw_name Size>]
                        ) + std::mem::size_of::<usize>(),
                    "array length and pointer must be adjacent"
                );
                assert!(
                    std::mem::size_of::<usize>()
                        == std::mem::size_of::<*mut std::ffi::c_void>()
                );
            };

            #[cfg(test)]
            #[test]
            fn [<verify_ $name:snake _ $raw_name:snake _array_metadata>]() {
                $crate::data_type::verify_array_member::<$name, $arr_t>(
                    std::mem::offset_of!(
                        <$name as $crate::DataType>::Inner,
                        [<$raw_name Size>]
                    ),
                    std::mem::offset_of!(<$name as $crate::DataType>::Inner, $raw_name),
                );
            }
        }
    };

    // Validation is independent of the two method-generation walks.
    (@checks $name:ident; #[doc = $doc:expr] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@checks $name; $($fields)+);
    };
    (@checks $name:ident; #[builder(skip)] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@checks $name; $($fields)+);
    };
    (@checks $name:ident; #[skip(uses = $function:path)] $raw_name:ident: $($rest:tt)+) => {
        paste::paste! {
            impl $name {
                const [<__ $raw_name:upper _OVERRIDE>]: () = { let _ = $function; };
            }
        }
        $crate::data_type::ua_struct!(@checks $name; $raw_name: $($rest)+);
    };
    (@checks $name:ident; #[skip] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@checks $name; $($fields)+);
    };
    (@checks $name:ident; $raw_name:ident: [$element:ty] $(, $($rest:tt)+)?) => {
        $crate::data_type::ua_struct!(@array_check $name, $raw_name, $element);
        $($crate::data_type::ua_struct!(@checks $name; $($rest)+);)?
    };
    (@checks $name:ident; #[from_inner] $raw_name:ident: $wrapper:ty $(, $($rest:tt)+)?) => {
        $crate::data_type::ua_struct!(@checks $name; $raw_name: &$wrapper);
        $($crate::data_type::ua_struct!(@checks $name; $($rest)+);)?
    };
    (@checks $name:ident; #[enum_copy] $raw_name:ident: $wrapper:ty $(, $($rest:tt)+)?) => {
        const _: fn(&<$name as $crate::DataType>::Inner) -> $wrapper = |raw| {
            // SAFETY: This closure is type-checked, never executed.
            unsafe { <$wrapper>::from_ua_unchecked(&raw.$raw_name) }
        };
        $($crate::data_type::ua_struct!(@checks $name; $($rest)+);)?
    };
    (@checks $name:ident; $raw_name:ident: &mut $wrapper:ty $(, $($rest:tt)+)?) => {
        $crate::data_type::ua_struct!(@checks $name; $raw_name: &$wrapper);
        $($crate::data_type::ua_struct!(@checks $name; $($rest)+);)?
    };
    (@checks $name:ident; $raw_name:ident: &$wrapper:ty $(, $($rest:tt)+)?) => {
        const _: fn(&<$name as $crate::DataType>::Inner) -> &<$wrapper as $crate::DataType>::Inner =
            |raw| &raw.$raw_name;
        $($crate::data_type::ua_struct!(@checks $name; $($rest)+);)?
    };
    (@checks $name:ident; $raw_name:ident: $storage:ty $(, $($rest:tt)+)?) => {
        const _: fn(&<$name as $crate::DataType>::Inner) -> &$storage = |raw| &raw.$raw_name;
        $($crate::data_type::ua_struct!(@checks $name; $($rest)+);)?
    };

    (@fields #[doc = $doc:expr] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@fields $($fields)+);
    };

    (@fields #[builder(skip)] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@fields $($fields)+);
    };

    (@builders [$($docs:tt)*] #[doc = $doc:expr] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@builders [$($docs)* #[doc = $doc]] $($fields)+);
    };

    (@builders [$($docs:tt)*] #[builder(skip)] $(#[$meta:meta])* $raw_name:ident: $field:ty $(, $($rest:tt)+)?) => {
        $($crate::data_type::ua_struct!(@builders [] $($rest)+);)?
    };

    (@builders [$($docs:tt)*] #[skip$((uses = $function:path))?] $($fields:tt)+) => {
        $crate::data_type::ua_struct!(@builders [$($docs)*] $($fields)+);
    };

    (@builders [$($docs:tt)*] $raw_name:ident: [$element:ty] $(, $($rest:tt)+)?) => {
        paste::paste! {
            $($docs)*
            /// Replaces the array, taking ownership of its buffer and elements.
            #[must_use]
            pub fn [<with_ $raw_name:snake>](mut self, value: $crate::ua::Array<$element>) -> Self {
                value.move_into_raw(
                    &mut self.0.[<$raw_name Size>],
                    &mut self.0.$raw_name,
                );
                self
            }
        }

        $($crate::data_type::ua_struct!(@builders [] $($rest)+);)?
    };

    (@builders [$($docs:tt)*] $raw_name:ident: &mut $wrapper:ty $(, $($rest:tt)+)?) => {
        $crate::data_type::ua_struct!(@builders [$($docs)*] $raw_name: &$wrapper);
        $($crate::data_type::ua_struct!(@builders [] $($rest)+);)?
    };

    (@builders [$($docs:tt)*] $raw_name:ident: &$wrapper:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            $($docs)*
            /// Replaces the field without cloning its contents.
            #[must_use]
            pub fn [<with_ $raw_name:snake>](mut self, value: $wrapper) -> Self {
                $crate::DataType::move_into_raw(value, &mut self.0.$raw_name);
                self
            }
        }

        $($crate::data_type::ua_struct!(@builders [] $($rest)+);)?
    };

    (@builders [$($docs:tt)*] #[from_inner] $raw_name:ident: $wrapper:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            $($docs)*
            #[must_use]
            pub const fn [<with_ $raw_name:snake>](mut self, value: $wrapper) -> Self {
                *$crate::data_type::data_type_ref_mut::<$wrapper>(&mut self.0.$raw_name) = value;
                self
            }
        }

        $($crate::data_type::ua_struct!(@builders [] $($rest)+);)?
    };

    (@builders [$($docs:tt)*] $raw_name:ident: $primitive:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            $($docs)*
            #[must_use]
            pub const fn [<with_ $raw_name:snake>](mut self, value: $primitive) -> Self {
                self.0.$raw_name = value;
                self
            }
        }

        $($crate::data_type::ua_struct!(@builders [] $($rest)+);)?
    };

    (@fields $raw_name:ident: [$arr_t:ty] $(, $($rest:tt)+)?) => {
        paste::paste! {
            /// Borrows the array elements. Returns `None` for an undefined array.
            #[must_use]
            pub fn [<$raw_name:snake>](&self) -> Option<&[$arr_t]> {
                // SAFETY: The declaration matches a non-optional C array member.
                unsafe {
                    $crate::ua::Array::slice_from_raw_parts(
                        self.0.[<$raw_name Size>],
                        self.0.$raw_name,
                    )
                }
            }

            /// Borrows the array elements mutably. Returns `None` for an undefined array.
            #[must_use]
            pub fn [<$raw_name:snake _mut>](&mut self) -> Option<&mut [$arr_t]> {
                // SAFETY: The declared element layout matches, and &mut self is exclusive.
                unsafe {
                    $crate::ua::Array::slice_from_raw_parts_mut(
                        self.0.[<$raw_name Size>],
                        self.0.$raw_name,
                    )
                }
            }

            /// Takes ownership of the array elements. Returns `None` for an undefined array.
            ///
            /// Selection and presence bits elsewhere in this value are left unchanged.
            #[must_use]
            pub fn [<take_ $raw_name:snake>](&mut self) -> Option<$crate::ua::Array<$arr_t>> {
                // SAFETY: The member is a non-optional array, and &mut self is exclusive.
                unsafe {
                    $crate::ua::Array::move_from_raw_parts(
                        &mut self.0.[<$raw_name Size>],
                        &mut self.0.$raw_name,
                    )
                }
            }
        }

        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };

    // The validation walk checks skipped declarations; this walk only suppresses accessors.
    (@fields #[skip$((uses = $fn_name:path))?] $raw_name:ident: $field_t:ty $(, $($rest:tt)+)?) => {
        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };

    // create an immutable reference accessor method for the field
    (@fields $raw_name:ident: &$wrapper_t:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            pub const fn [<$raw_name:snake>](&self) -> &$wrapper_t {
                $crate::data_type::data_type_ref(&self.0.$raw_name)
            }
        }

        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };

    // create both an immutable _and_ mutable reference accessor for the field
    (@fields $raw_name:ident: &mut $wrapper_t:ty $(, $($rest:tt)+)?) => {
        $crate::data_type::ua_struct!(@fields $raw_name: &$wrapper_t);

        paste::paste! {
            pub const fn [<$raw_name:snake _mut>](&mut self) -> &mut $wrapper_t {
                $crate::data_type::data_type_ref_mut(&mut self.0.$raw_name)
            }
        }

        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };

    // create a by-value copy of `$wrapper_t` with an implicit copy of its inner type
    (@fields #[from_inner] $raw_name:ident: $wrapper_t:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            pub const fn [<$raw_name:snake>](&self) -> $wrapper_t {
                *$crate::data_type::data_type_ref(&self.0.$raw_name)
            }
        }

        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };

    // Copy an enum when the wrapper uses a different, proven native integer representation.
    (@fields #[enum_copy] $raw_name:ident: $wrapper_t:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            pub const fn [<$raw_name:snake>](&self) -> $wrapper_t {
                // SAFETY: The helper verifies and copies the bindgen enum's integer bits.
                unsafe { $wrapper_t::from_ua_unchecked(&self.0.$raw_name) }
            }
        }

        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };

    // create a by-value copy of the field that doesn't have a wrapper
    (@fields $raw_name:ident: $field_t:ty $(, $($rest:tt)+)?) => {
        paste::paste! {
            pub const fn [<$raw_name:snake>](&self) -> $field_t {
                self.0.$raw_name
            }
        }

        $($crate::data_type::ua_struct!(@fields $($rest)+);)?
    };
}
pub(crate) use ua_struct;

/// Declares a scalar wrapper using its native primitive representation.
macro_rules! ua_primitive {
    (pub struct Float;) => {
        $crate::data_type::ua_primitive!(@float Float);
    };
    (pub struct Double;) => {
        $crate::data_type::ua_primitive!(@float Double);
    };
    (@float $name:ident) => {
        $crate::data_type::ua_primitive!(@storage $name;);
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0 || (self.0.is_nan() && other.0.is_nan())
            }
        }
        impl Eq for $name {}
    };
    (pub struct $name:ident;) => {
        $crate::data_type::ua_primitive!(@storage $name; PartialEq, Eq);
        #[cfg(test)]
        const _: () = {
            // SAFETY: Primitive storage accepts its native zero initializer.
            const ZERO: $name = unsafe { std::mem::zeroed() };
            let _: fn($name) -> bool = |value| matches!(value, ZERO);
        };
    };
    (@storage $name:ident; $($equality:ident),*) => {
        paste::paste! {
            #[doc = concat!("Primitive wrapper for `", stringify!($name), "`.")]
            #[repr(transparent)]
            #[derive(Clone, Copy $(, $equality)*)]
            pub struct $name(open62541_sys::[<UA_ $name>]);

            $crate::data_type::native_type_impl!($name, [<UA_ $name>], [<UA_TYPES_ $name:upper>]);
            #[cfg(test)]
            #[test]
            fn [<verify_ $name:snake _primitive_layout>]() {
                $crate::data_type::verify_scalar_layout::<$name>(
                    u32::from_ne_bytes(open62541_sys::UA_DataTypeKind::[<UA_DATATYPEKIND_ $name:upper>].0.to_ne_bytes()),
                    std::mem::size_of::<open62541_sys::[<UA_ $name>]>(),
                    std::mem::align_of::<open62541_sys::[<UA_ $name>]>(),
                );
            }
        }
    };

    ($(pub struct $name:ident($raw:ident);)+) => {
        $(
            $crate::data_type::ua_primitive!(pub struct $name;);

            impl $name {
                #[must_use]
                pub const fn new(value: $raw) -> Self {
                    Self(value)
                }

                #[must_use]
                pub const fn value(&self) -> $raw {
                    self.0
                }
            }

            #[cfg(feature = "serde")]
            impl serde::Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    paste::paste! {
                        serializer.[<serialize_ $raw>](self.0)
                    }
                }
            }
        )+


    };
}
pub(crate) use ua_primitive;

/// Declares a semantic wrapper over a known primitive or allocating builtin.
#[cfg_attr(
    not(test),
    expect(
        unused_macro_rules,
        reason = "String aliases are currently exercised by a private test declaration"
    )
)]
macro_rules! ua_alias {
    (#[serialize_as(ByteString)] pub struct $name:ident;) => {
        $crate::data_type::ua_alias!(@allocating $name, ByteString);
    };

    (#[serialize_as(String)] pub struct $name:ident;) => {
        $crate::data_type::ua_alias!(@allocating $name, String);
    };

    (@allocating $name:ident, $repr:ident) => {
        paste::paste! {
            #[doc = concat!("Owned wrapper for `open62541_sys::UA_", stringify!($name), "`.")]
            #[repr(transparent)]
            pub struct $name(open62541_sys::[<UA_ $name>]);

            $crate::data_type::native_type_impl!(
                $name,
                [<UA_ $name>],
                [<UA_TYPES_ $name:upper>], native_eq;
                fn accepts_variant_type(data_type: *const open62541_sys::UA_DataType) -> bool {
                    // The bindings expose the same C storage, including ownership rules.
                    let _: fn(&open62541_sys::[<UA_ $repr>]) -> &Self::Inner = |value| value;
                    data_type == Self::data_type()
                        || data_type == <$crate::ua::$repr as $crate::DataType>::data_type()
                }
            );
            $crate::data_type::owned_lifecycle!($name);
        }
    };

    (#[serialize_as($inner:ident)] $(#[$meta:meta])* pub struct $name:ident;) => {
        paste::paste! {
            #[doc = concat!("Native alias wrapper for `", stringify!($name), "`.")]
            #[repr(transparent)]
            #[derive(Clone, Copy, PartialEq, Eq)]
            $(#[$meta])*
            pub struct $name(open62541_sys::[<UA_ $inner>]);

            $crate::data_type::native_type_impl!($name, [<UA_ $inner>], [<UA_TYPES_ $inner:upper>]);
            #[cfg(test)]
            const _: () = {
                // SAFETY: Scalar aliases retain their primitive zero initializer.
                const ZERO: $name = unsafe { std::mem::zeroed() };
                let _: fn($name) -> bool = |value| matches!(value, ZERO);
            };
        }
    };
}
pub(crate) use ua_alias;

/// Checks an array declaration against the linked C schema in generated tests.
#[cfg(test)]
pub(crate) fn verify_array_member<T: DataType, Element: DataType>(
    length_offset: usize,
    pointer_offset: usize,
) {
    // SAFETY: DataType implementations return a valid static descriptor and member table.
    let descriptor = unsafe { &*T::data_type() };
    verify_array_metadata::<Element>(descriptor, length_offset, pointer_offset);
}

#[cfg(test)]
fn verify_array_metadata<Element: DataType>(
    descriptor: &UA_DataType,
    length_offset: usize,
    pointer_offset: usize,
) {
    let kind = u32::from_ne_bytes(
        open62541_sys::UA_DataTypeKind::UA_DATATYPEKIND_STRUCTURE
            .0
            .to_ne_bytes(),
    );
    assert_eq!(
        descriptor.typeKind(),
        kind,
        "array accessor requires a structure"
    );
    let count = usize::try_from(descriptor.membersSize()).expect("member count fits usize");
    assert!(
        count > 0 && !descriptor.members.is_null(),
        "structure has no metadata"
    );
    // SAFETY: The descriptor's member table contains membersSize initialized entries.
    let members = unsafe { std::slice::from_raw_parts(descriptor.members, count) };
    let mut offset = 0usize;
    for member in members {
        offset = offset
            .checked_add(usize::from(member.padding()))
            .expect("member offset overflow");
        if offset == length_offset {
            assert_eq!(member.isArray(), 1, "declared field is not an array");
            assert_eq!(pointer_offset, offset + std::mem::size_of::<usize>());
            assert_eq!(
                member.isOptional(),
                0,
                "optional arrays require explicit handling"
            );
            assert!(
                Element::accepts_variant_type(member.memberType),
                "incompatible array element descriptor"
            );
            return;
        }

        let size = if member.isArray() != 0 {
            2 * std::mem::size_of::<usize>()
        } else if member.isOptional() != 0 {
            std::mem::size_of::<*mut c_void>()
        } else {
            // SAFETY: A structure member's type descriptor is valid and static.
            let member_type = unsafe { member.memberType.as_ref() }.expect("member type");
            usize::try_from(member_type.memSize()).expect("member size fits usize")
        };
        offset = offset.checked_add(size).expect("member offset overflow");
    }
    panic!("array field is absent from the structure metadata");
}

#[cfg(test)]
mod skipped_array_checks {
    mod neither_skipped {
        use crate::ua;
        type Check = ua::DeleteSubscriptionsRequest;
        crate::data_type::ua_struct!(@checks Check; subscriptionIds: [ua::UInt32]);
    }
    mod getter_skipped {
        use crate::ua;
        type Check = ua::DeleteSubscriptionsRequest;
        crate::data_type::ua_struct!(@checks Check; #[skip] subscriptionIds: [ua::UInt32]);
    }
    mod builder_skipped {
        use crate::ua;
        type Check = ua::DeleteSubscriptionsRequest;
        crate::data_type::ua_struct!(@checks Check; #[builder(skip)] subscriptionIds: [ua::UInt32]);
    }
    mod both_skipped {
        use crate::ua;
        type Check = ua::DeleteSubscriptionsRequest;
        crate::data_type::ua_struct!(@checks Check;
            #[builder(skip)] #[skip] subscriptionIds: [ua::UInt32]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn floating_equality_preserves_native_nan_and_signed_zero_rules() {
        use crate::ua;
        macro_rules! check {
            ($wrapper:ident, $raw:ty) => {{
                let nan = ua::$wrapper::new(<$raw>::NAN);
                let other_nan = ua::$wrapper::new(-<$raw>::NAN);
                let zero = ua::$wrapper::new(0.0);
                let negative_zero = ua::$wrapper::new(-0.0);
                assert_eq!(nan, other_nan);
                assert_ne!(nan, zero);
                assert_eq!(zero, negative_zero);
                assert_eq!(
                    <ua::$wrapper as crate::DataType>::order(&nan, &other_nan),
                    open62541_sys::UA_Order::UA_ORDER_EQ
                );
            }};
        }
        check!(Float, f32);
        check!(Double, f64);
    }

    #[test]
    fn enum_ordering_retains_unsigned_high_bit_rule() {
        use crate::ua;
        // SAFETY: Every bit pattern is valid native enum storage.
        let high = unsafe { ua::NodeClass::from_u32_unchecked(u32::MAX) };
        assert_eq!(high, high.clone());
        assert_eq!(
            high.cmp(&ua::NodeClass::UNSPECIFIED),
            std::cmp::Ordering::Greater
        );
    }

    #[expect(
        unreachable_pub,
        reason = "private test declaration for allocating String aliases"
    )]
    mod string_alias {
        use crate::ua;

        crate::data_type::ua_alias! {
            #[serialize_as(String)]
            pub struct LocaleId;
        }

        #[test]
        fn string_alias_owns_and_decodes_both_representations() {
            let source = ua::String::new("en-US").unwrap();
            let wire = ua::Variant::scalar(source.clone());
            let alias = wire.try_into_scalar::<LocaleId>().unwrap();
            let clone = alias.clone();
            drop(alias);

            let native = ua::Variant::scalar(clone);
            assert_eq!(native.as_scalar::<LocaleId>().unwrap().0.length, 5);
            let recovered = native.try_into_scalar::<LocaleId>().unwrap();
            let array = ua::Variant::array(ua::Array::from_slice(std::slice::from_ref(&source)));
            let aliases = array.to_array::<LocaleId>().unwrap();
            assert_eq!(aliases[0], recovered);
            assert_eq!(source.as_str(), Some("en-US"));
        }
    }

    #[test]
    fn array_metadata_rejects_optional_and_discriminated_storage() {
        use super::{DataType, verify_array_metadata};
        use crate::ua;
        use open62541_sys::{UA_DataType, UA_DataTypeKind, UA_DataTypeMember};

        let mut member = UA_DataTypeMember {
            memberType: ua::UInt32::data_type(),
            ..Default::default()
        };
        member.set_isArray(1);
        let mut descriptor = UA_DataType {
            members: &raw mut member,
            ..Default::default()
        };
        descriptor.set_typeKind(u32::from_ne_bytes(
            UA_DataTypeKind::UA_DATATYPEKIND_STRUCTURE.0.to_ne_bytes(),
        ));
        descriptor.set_membersSize(1);
        let pointer_offset = std::mem::size_of::<usize>();
        verify_array_metadata::<ua::UInt32>(&descriptor, 0, pointer_offset);

        member.set_isOptional(1);
        assert!(
            std::panic::catch_unwind(|| {
                verify_array_metadata::<ua::UInt32>(&descriptor, 0, pointer_offset);
            })
            .is_err()
        );
        member.set_isOptional(0);

        descriptor.set_typeKind(u32::from_ne_bytes(
            UA_DataTypeKind::UA_DATATYPEKIND_UNION.0.to_ne_bytes(),
        ));
        assert!(
            std::panic::catch_unwind(|| {
                verify_array_metadata::<ua::UInt32>(&descriptor, 0, pointer_offset);
            })
            .is_err()
        );
        descriptor.set_typeKind(u32::from_ne_bytes(
            UA_DataTypeKind::UA_DATATYPEKIND_STRUCTURE.0.to_ne_bytes(),
        ));
        assert!(
            std::panic::catch_unwind(|| {
                verify_array_metadata::<ua::Byte>(&descriptor, 0, pointer_offset);
            })
            .is_err()
        );
    }

    use std::thread;

    use super::*;

    #[test]
    fn array_fields_borrow_mutate_and_transfer_the_same_storage() {
        let initial = ua::DataValue::new(ua::Variant::scalar(ua::String::new("before").unwrap()));
        let results: ua::Array<_> = [initial].into();
        let original = results.as_slice().as_ptr();
        let mut response = ua::ReadResponse::init().with_results(results);

        let pointer = response.results().unwrap().as_ptr();
        assert_eq!(pointer, original);
        *response.results_mut().unwrap().first_mut().unwrap() =
            ua::DataValue::new(ua::Variant::scalar(ua::String::new("after").unwrap()));

        let results = response.take_results().unwrap();
        assert_eq!(results.as_slice().as_ptr(), pointer);
        assert!(response.results().is_none());
        drop(response);

        let value = results.as_slice().first().unwrap().value().unwrap();
        assert_eq!(
            value.as_scalar::<ua::String>().unwrap().as_str(),
            Some("after"),
        );
    }

    #[test]
    fn generated_builders_move_owned_fields_and_replace_previous_storage() {
        let name = ua::String::new("first").unwrap();
        let first_pointer = name.as_bytes().unwrap().as_ptr();
        let argument = ua::Argument::init().with_name(name);
        assert_eq!(argument.name().as_bytes().unwrap().as_ptr(), first_pointer);

        let replacement = ua::String::new("replacement").unwrap();
        let replacement_pointer = replacement.as_bytes().unwrap().as_ptr();
        let argument = argument.with_name(replacement);
        assert_eq!(
            argument.name().as_bytes().unwrap().as_ptr(),
            replacement_pointer
        );
        assert_eq!(argument.name().as_str(), Some("replacement"));

        let payload = ua::String::new("array element").unwrap();
        let payload_pointer = payload.as_bytes().unwrap().as_ptr();
        let array: ua::Array<_> = vec![payload].into();
        let array_pointer = array.as_slice().as_ptr();
        let description = ua::ApplicationDescription::init().with_discovery_urls(array);
        let values = description.discovery_urls().unwrap();
        assert_eq!(values.as_ptr(), array_pointer);
        assert_eq!(
            values.first().unwrap().as_bytes().unwrap().as_ptr(),
            payload_pointer
        );

        let payload = ua::String::new("scalar replacement").unwrap();
        let payload_pointer = payload.as_bytes().unwrap().as_ptr();
        let variant = ua::Variant::scalar(ua::UInt32::new(7)).with_scalar(payload);
        assert_eq!(
            variant
                .as_scalar::<ua::String>()
                .unwrap()
                .as_bytes()
                .unwrap()
                .as_ptr(),
            payload_pointer,
        );
    }

    #[test]
    fn scalar_builder_assignments_remain_const() {
        const fn configure(value: ua::ReadValueId) -> ua::ReadValueId {
            value.with_attribute_id(ua::AttributeId::NODECLASS)
        }
        const fn configure_argument(value: ua::Argument) -> ua::Argument {
            value.with_value_rank(-1)
        }
        assert_eq!(
            configure(ua::ReadValueId::init()).attribute_id(),
            ua::AttributeId::NODECLASS
        );
        assert_eq!(configure_argument(ua::Argument::init()).value_rank(), -1);
    }

    #[test]
    fn custom_builders_keep_presence_and_specified_flags_with_owned_payloads() {
        use crate::Attributes as _;

        let value = ua::Variant::scalar(ua::UInt32::new(42));
        let data = ua::DataValue::init()
            .with_value(value)
            .with_status(ua::StatusCode::BADINTERNALERROR);
        assert_eq!(
            data.value()
                .unwrap()
                .as_scalar::<ua::UInt32>()
                .unwrap()
                .value(),
            42
        );
        assert_eq!(data.status(), Some(ua::StatusCode::BADINTERNALERROR));

        let mut trust = ua::TrustListDataType::init()
            .with_trusted_certificates([ua::ByteString::new(b"certificate")].into());
        assert_eq!(
            trust.specified_lists(),
            ua::TrustListMasks::TRUSTEDCERTIFICATES_U32
        );
        trust = trust.with_trusted_certificates([].into());
        assert_eq!(trust.specified_lists(), 0);
        assert_eq!(trust.trusted_certificates(), Some([].as_slice()));

        let attributes = ua::VariableAttributes::default()
            .with_data_type(ua::NodeId::ns0(open62541_sys::UA_NS0ID_STRING));
        assert_ne!(
            attributes.as_node_attributes().specified_attributes()
                & ua::SpecifiedAttributes::DATATYPE.as_u32(),
            0
        );
    }

    #[test]
    fn send_sync_string() {
        let string = ua::String::new("Lorem Ipsum").expect("create string");

        // References to string can be accessed in different threads.
        thread::scope(|scope| {
            scope.spawn(|| {
                let _ = &string;
            });
        });

        // Ownership of string can be passed to different thread.
        thread::spawn(move || {
            drop(string);
        })
        .join()
        .expect("join thread");
    }
}
