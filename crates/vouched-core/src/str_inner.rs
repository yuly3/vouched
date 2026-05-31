#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, string::String, sync::Arc};

#[allow(
    private_bounds,
    reason = "the private supertrait is the sealing mechanism for supported string inner types"
)]
#[doc(hidden)]
pub trait VouchedStrInner: sealed::Sealed + AsRef<str> {
    fn from_validated_str(s: &str) -> Self;
}

mod sealed {
    pub trait Sealed {}
}

#[cfg(feature = "alloc")]
impl sealed::Sealed for String {}

#[cfg(feature = "alloc")]
impl VouchedStrInner for String {
    fn from_validated_str(s: &str) -> Self {
        Self::from(s)
    }
}

#[cfg(feature = "alloc")]
impl sealed::Sealed for Box<str> {}

#[cfg(feature = "alloc")]
impl VouchedStrInner for Box<str> {
    fn from_validated_str(s: &str) -> Self {
        Self::from(s)
    }
}

#[cfg(feature = "alloc")]
impl sealed::Sealed for Rc<str> {}

#[cfg(feature = "alloc")]
impl VouchedStrInner for Rc<str> {
    fn from_validated_str(s: &str) -> Self {
        Self::from(s)
    }
}

#[cfg(feature = "alloc")]
impl sealed::Sealed for Arc<str> {}

#[cfg(feature = "alloc")]
impl VouchedStrInner for Arc<str> {
    fn from_validated_str(s: &str) -> Self {
        Self::from(s)
    }
}
