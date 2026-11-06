mod variable_attributes;

use open62541_sys::{UA_DataType, UA_NodeAttributes};

use crate::{DataType as _, ua};

crate::data_type::ua_struct! {
    pub struct NodeAttributes {
        #[builder(skip)]
        description: &ua::LocalizedText,
        #[builder(skip)]
        displayName: &ua::LocalizedText,
        #[builder(skip)]
        #[from_inner]
        writeMask: ua::AttributeWriteMask,
        #[builder(skip)]
        #[from_inner]
        userWriteMask: ua::AttributeWriteMask,
        #[builder(skip)]
        specifiedAttributes: u32
    }
}

macro_rules! derived {
    ($( $name:ident $( { $($fields:tt)* } )? ),* $(,)?) => {
        $(
            paste::paste! {
                $crate::data_type::ua_struct! {
                    pub struct [<$name Attributes>] {
                        $($($fields)*)?
                    }
                }
            }

            impl $crate::Attributes for paste::paste!{[<$name Attributes>]} {
                fn node_class(&self) -> ua::NodeClass {
                    paste::paste! {
                        ua::NodeClass::[<$name:upper>]
                    }
                }

                fn attribute_type(&self) -> *const UA_DataType {
                    <Self as $crate::DataType>::data_type()
                }

                fn with_display_name(mut self, display_name: ua::LocalizedText) -> Self {
                    // SAFETY: These attributes exclusively own the initialized display name.
                    unsafe { display_name.move_into_raw(&mut self.0.displayName) };
                    self.0.specifiedAttributes |= ua::SpecifiedAttributes::DISPLAYNAME.as_u32();
                    self
                }

                fn as_node_attributes(&self) -> &ua::NodeAttributes {
                    // SAFETY: This transmutes from `Self` to `UA_NodeAttributes`, a strict subset of
                    // `UA_(...)Attributes` with the same memory layout.
                    let node_attributes = unsafe { self.as_ptr().cast::<UA_NodeAttributes>() };
                    // SAFETY: Transmutation is allowed and pointer is valid (non-zero).
                    let node_attributes = unsafe { node_attributes.as_ref().unwrap_unchecked() };
                    ua::NodeAttributes::raw_ref(node_attributes)
                }
            }

            impl Default for paste::paste!{[<$name Attributes>]} {
                fn default() -> Self {
                    paste::paste! {
                        Self::clone_raw(unsafe { &open62541_sys::[<UA_ $name Attributes_default>] })
                    }
                }
            }
        )*
    };
}

// This adds basic declarations and shared functionality such as upcasting to `ua::NodeAttributes`.
// See sub-modules for type-specific implementations, e.g. `variable_attributes`.
derived!(
    Object,
    Variable {
        #[builder(skip)]
        dataType: &ua::NodeId,
        #[builder(skip)]
        valueRank: i32,
        #[builder(skip)]
        arrayDimensions: [ua::UInt32],
        #[builder(skip)]
        #[from_inner]
        accessLevel: ua::AccessLevelType
    },
    Method {
        #[builder(skip)]
        executable: bool,
        #[builder(skip)]
        userExecutable: bool
    },
    ObjectType,
    VariableType,
    ReferenceType,
    DataType,
    View,
    // Generic, // Omitted for now because the `Default` impl above cannot be used here.
);

impl MethodAttributes {
    #[must_use]
    pub const fn with_executable(mut self, executable: bool) -> Self {
        self.0.executable = executable;
        self.0.specifiedAttributes |= ua::SpecifiedAttributes::EXECUTABLE.as_u32();
        self
    }

    #[must_use]
    pub const fn with_user_executable(mut self, user_executable: bool) -> Self {
        self.0.userExecutable = user_executable;
        self.0.specifiedAttributes |= ua::SpecifiedAttributes::USEREXECUTABLE.as_u32();
        self
    }
}
