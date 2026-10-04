use std::fmt::Debug;

use open62541_sys::UA_DataType;

use crate::{DataType, Result, ua};

/// Extended values.
///
/// This supports semantic values that convert to and from a native [`DataType`]. Native data
/// types also implement this trait directly, with themselves as the intermediate type.
pub trait DataTypeExt: Debug + Clone {
    /// Native type used as the intermediary for extraction and conversion.
    ///
    /// This is not necessarily the descriptor produced by a wire decoder.
    type Inner: DataType;

    /// Creates instance for inner type.
    ///
    /// # Errors
    ///
    /// This may fail when the conversion from the inner type cannot be done.
    fn from_inner(value: Self::Inner) -> Result<Self>;

    /// Returns inner type representation.
    fn into_inner(self) -> Self::Inner;
}

// Umbrella implementation that simplifies type constraints: `DataType` is trivially `DataTypeExt`.
impl<T: DataType> DataTypeExt for T {
    type Inner = Self;

    fn from_inner(value: Self::Inner) -> Result<Self> {
        Ok(value)
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

/// Node attribute.
///
/// This is used to match the appropriate result types at compile time when reading attributes from
/// nodes. See the following methods for details:
///
/// - [`AsyncClient::read_attribute()`](crate::AsyncClient::read_attribute)
/// - [`Server::read_attribute()`](crate::Server::read_attribute)
//
// FIXME: Turn into sealed trait.
pub trait Attribute: Debug + Copy {
    /// Attribute data type.
    type Value: DataTypeExt + Send;

    /// Gets attribute ID.
    fn id(&self) -> ua::AttributeId;
}

/// Server node attributes.
///
/// This is used to allow handling different node types when adding nodes to the server's data tree
/// in [`Server::add_node()`](crate::Server::add_node).
//
// FIXME: Turn into sealed trait.
pub trait Attributes: DataType {
    /// Gets associated node class.
    fn node_class(&self) -> ua::NodeClass;

    /// Gets associated attribute type.
    ///
    /// This is [`<Self as DataType>::data_type()`] with a more appropriate name.
    ///
    /// [`<Self as DataType>::data_type()`]: DataType::data_type()
    fn attribute_type(&self) -> *const UA_DataType;

    /// Sets display name.
    #[must_use]
    fn with_display_name(self, display_name: ua::LocalizedText) -> Self;

    /// Gets generic [`ua::NodeAttributes`] type.
    fn as_node_attributes(&self) -> &ua::NodeAttributes;
}

/// Custom certificate verification.
///
/// This is used to implement custom callbacks in [`ua::CertificateVerification::custom()`].
pub trait CustomCertificateVerification {
    fn verify_certificate(&self, certificate: &ua::ByteString) -> ua::StatusCode;
}

/// Private-key password callback.
///
/// This is used to fetch the password for a given client private key when establishing a connection
/// in [`Client`] or [`AsyncClient`].
///
/// See [`ClientBuilder::private_key_password_callback()`] for details.
///
/// [`Client`]: crate::Client
/// [`AsyncClient`]: crate::AsyncClient
/// [`ClientBuilder::private_key_password_callback()`]: crate::ClientBuilder::private_key_password_callback
#[cfg(feature = "mbedtls")]
pub trait PrivateKeyPasswordCallback {
    /// Provides private-key password.
    ///
    /// # Errors
    ///
    /// This should return an appropriate error when the password cannot be provided.
    fn private_key_password(&self) -> Result<crate::Password>;
}

#[cfg(feature = "mbedtls")]
impl<F> PrivateKeyPasswordCallback for F
where
    F: Fn() -> Result<crate::Password>,
{
    fn private_key_password(&self) -> Result<crate::Password> {
        self()
    }
}

/// Monitoring filter.
///
/// Convert a filter with [`MonitoringFilter::to_extension_object()`] before passing the owned
/// object to [`ua::MonitoringParameters::with_filter()`].
pub trait MonitoringFilter: Debug + Send + Sync + 'static {
    fn to_extension_object(&self) -> ua::ExtensionObject;
}

impl MonitoringFilter for Box<dyn MonitoringFilter> {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        (**self).to_extension_object()
    }
}

/// Filter operand.
///
/// Convert operands with [`FilterOperand::to_extension_object()`], collect the owned objects into
/// a [`ua::Array`], and pass it to [`ua::ContentFilterElement::with_filter_operands()`].
pub trait FilterOperand: Debug + Send + Sync + 'static {
    fn to_extension_object(&self) -> ua::ExtensionObject;
}

impl FilterOperand for Box<dyn FilterOperand> {
    fn to_extension_object(&self) -> ua::ExtensionObject {
        (**self).to_extension_object()
    }
}
