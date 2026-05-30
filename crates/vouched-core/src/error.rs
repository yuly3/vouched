use core::{error::Error as StdError, fmt};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use core::ops::Deref;

#[cfg(feature = "valuable")]
use alloc::string::ToString;
#[cfg(feature = "valuable")]
use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

/// Error returned when a string newtype is shorter than its `len` lower bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooShortError {
    min: usize,
    actual: usize,
}

impl TooShortError {
    /// Creates a too-short error.
    pub const fn new(min: usize, actual: usize) -> Self {
        Self { min, actual }
    }

    /// Returns the minimum accepted length, measured as untrimmed Unicode scalar values.
    pub const fn min(&self) -> usize {
        self.min
    }

    /// Returns the actual length, measured as untrimmed Unicode scalar values.
    pub const fn actual(&self) -> usize {
        self.actual
    }
}

impl fmt::Display for TooShortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "is too short (min {} chars, got {})",
            self.min, self.actual
        )
    }
}

impl StdError for TooShortError {}

/// Error returned when a string newtype is longer than its `len` upper bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooLongError {
    max: usize,
    actual: usize,
}

impl TooLongError {
    /// Creates a too-long error.
    pub const fn new(max: usize, actual: usize) -> Self {
        Self { max, actual }
    }

    /// Returns the maximum accepted length, measured as untrimmed Unicode scalar values.
    pub const fn max(&self) -> usize {
        self.max
    }

    /// Returns the actual length, measured as untrimmed Unicode scalar values.
    pub const fn actual(&self) -> usize {
        self.actual
    }
}

impl fmt::Display for TooLongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "is too long (max {} chars, got {})",
            self.max, self.actual
        )
    }
}

impl StdError for TooLongError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NumericValueRepr {
    Signed(i128),
    Unsigned(u128),
}

/// Lossless numeric value captured by generated range and cast errors.
///
/// The representation is private so future integer-like values can be added
/// without exposing the enum shape. Use the `as_*` methods to recover a
/// primitive integer only when the conversion is lossless.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NumericValue(NumericValueRepr);

impl NumericValue {
    /// Captures a signed integer value.
    pub const fn from_i128(value: i128) -> Self {
        Self(NumericValueRepr::Signed(value))
    }

    /// Captures an unsigned integer value.
    pub const fn from_u128(value: u128) -> Self {
        Self(NumericValueRepr::Unsigned(value))
    }

    /// Returns the value as `i64` when it fits exactly.
    pub fn as_i64(self) -> Option<i64> {
        match self.0 {
            NumericValueRepr::Signed(value) => i64::try_from(value).ok(),
            NumericValueRepr::Unsigned(value) => i64::try_from(value).ok(),
        }
    }

    /// Returns the value as `u64` when it fits exactly.
    pub fn as_u64(self) -> Option<u64> {
        match self.0 {
            NumericValueRepr::Signed(value) => u64::try_from(value).ok(),
            NumericValueRepr::Unsigned(value) => u64::try_from(value).ok(),
        }
    }

    /// Returns the value as `i128` when it fits exactly.
    pub fn as_i128(self) -> Option<i128> {
        match self.0 {
            NumericValueRepr::Signed(value) => Some(value),
            NumericValueRepr::Unsigned(value) => i128::try_from(value).ok(),
        }
    }

    /// Returns the value as `u128` when it fits exactly.
    pub fn as_u128(self) -> Option<u128> {
        match self.0 {
            NumericValueRepr::Signed(value) => u128::try_from(value).ok(),
            NumericValueRepr::Unsigned(value) => Some(value),
        }
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            NumericValueRepr::Signed(value) => write!(f, "{value}"),
            NumericValueRepr::Unsigned(value) => write!(f, "{value}"),
        }
    }
}

macro_rules! impl_signed_numeric_value_from {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for NumericValue {
                fn from(value: $ty) -> Self {
                    Self::from_i128(i128::from(value))
                }
            }
        )*
    };
}

macro_rules! impl_unsigned_numeric_value_from {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for NumericValue {
                fn from(value: $ty) -> Self {
                    Self::from_u128(u128::from(value))
                }
            }
        )*
    };
}

impl_signed_numeric_value_from!(i8, i16, i32, i64, i128);
impl_unsigned_numeric_value_from!(u8, u16, u32, u64, u128);

#[cfg(feature = "valuable")]
impl Valuable for NumericValue {
    fn as_value(&self) -> Value<'_> {
        match self.0 {
            NumericValueRepr::Signed(value) => Value::I128(value),
            NumericValueRepr::Unsigned(value) => Value::U128(value),
        }
    }

    fn visit(&self, visit: &mut dyn Visit) {
        visit.visit_value(self.as_value());
    }
}

/// Error returned when a numeric newtype is outside its `range` bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutOfRangeNumericError {
    actual: NumericValue,
    lower_bound: Option<NumericValue>,
    upper_bound: Option<NumericValue>,
}

impl OutOfRangeNumericError {
    /// Creates an out-of-range error with the value that failed validation.
    pub const fn new(actual: NumericValue) -> Self {
        Self {
            actual,
            lower_bound: None,
            upper_bound: None,
        }
    }

    /// Returns an out-of-range error with the lower bound that failed.
    #[must_use]
    pub const fn with_lower_bound(self, lower_bound: NumericValue) -> Self {
        Self {
            actual: self.actual,
            lower_bound: Some(lower_bound),
            upper_bound: self.upper_bound,
        }
    }

    /// Returns an out-of-range error with the upper bound that failed.
    #[must_use]
    pub const fn with_upper_bound(self, upper_bound: NumericValue) -> Self {
        Self {
            actual: self.actual,
            lower_bound: self.lower_bound,
            upper_bound: Some(upper_bound),
        }
    }

    /// Returns the actual value that failed validation.
    pub const fn actual(&self) -> NumericValue {
        self.actual
    }

    /// Returns the lower bound that failed when it could be captured losslessly.
    pub const fn lower_bound(&self) -> Option<NumericValue> {
        self.lower_bound
    }

    /// Returns the upper bound that failed when it could be captured losslessly.
    pub const fn upper_bound(&self) -> Option<NumericValue> {
        self.upper_bound
    }
}

impl fmt::Display for OutOfRangeNumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let actual = self.actual;
        match (self.lower_bound, self.upper_bound) {
            (Some(lower_bound), Some(upper_bound)) => {
                write!(
                    f,
                    "out of range (value {actual}, lower bound {lower_bound}, upper bound {upper_bound})"
                )
            }
            (Some(lower_bound), None) => {
                write!(
                    f,
                    "out of range (value {actual}, lower bound {lower_bound})"
                )
            }
            (None, Some(upper_bound)) => {
                write!(
                    f,
                    "out of range (value {actual}, upper bound {upper_bound})"
                )
            }
            (None, None) => write!(f, "out of range (value {actual})"),
        }
    }
}

impl StdError for OutOfRangeNumericError {}

#[cfg(feature = "valuable")]
static OUT_OF_RANGE_NUMERIC_ERROR_FIELDS: &[NamedField<'static>] = &[
    NamedField::new("actual"),
    NamedField::new("lower_bound"),
    NamedField::new("upper_bound"),
];

#[cfg(feature = "valuable")]
impl Valuable for OutOfRangeNumericError {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        let values = [
            self.actual.as_value(),
            self.lower_bound.as_value(),
            self.upper_bound.as_value(),
        ];
        visit.visit_named_fields(&NamedValues::new(
            OUT_OF_RANGE_NUMERIC_ERROR_FIELDS,
            &values,
        ));
    }
}

#[cfg(feature = "valuable")]
impl Structable for OutOfRangeNumericError {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_static(
            "OutOfRangeNumericError",
            Fields::Named(OUT_OF_RANGE_NUMERIC_ERROR_FIELDS),
        )
    }
}

/// Error returned when a string newtype contains a character rejected by `chars`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCharError {
    index: usize,
    ch: char,
}

impl InvalidCharError {
    /// Creates an invalid-character error.
    pub const fn new(index: usize, ch: char) -> Self {
        Self { index, ch }
    }

    /// Returns the zero-based character index, not a byte offset.
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the rejected character.
    pub const fn ch(&self) -> char {
        self.ch
    }
}

impl fmt::Display for InvalidCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "contains invalid character '{}' at index {}",
            self.ch, self.index
        )
    }
}

impl StdError for InvalidCharError {}

/// Common interface for generated Vouched errors.
///
/// Error types generated by the Vouched derive macro implement this trait.
/// The `as_*` methods provide type-safe access to specific validation errors.
pub trait VouchedError: StdError + Send + Sync + 'static {
    /// Returns the underlying too-short error when this is that variant.
    fn as_too_short(&self) -> Option<&TooShortError> {
        None
    }
    /// Returns the underlying too-long error when this is that variant.
    fn as_too_long(&self) -> Option<&TooLongError> {
        None
    }
    /// Returns the underlying numeric out-of-range error when this is that variant.
    fn as_out_of_range_numeric(&self) -> Option<&OutOfRangeNumericError> {
        None
    }
    /// Returns the underlying invalid-character error when this is that variant.
    fn as_invalid_char(&self) -> Option<&InvalidCharError> {
        None
    }
}

/// Wrapper type for handling different Vouched error types uniformly.
///
/// This allows code that works with multiple Vouched wrappers to handle their
/// validation errors through a single type. It supports automatic conversion
/// through the `?` operator.
///
/// Available with the `alloc` feature, which is enabled by the default `std`
/// feature.
///
/// # Examples
///
/// ```
/// # use vouched_core::*;
/// #
/// # #[derive(Debug)]
/// # enum UserIdError { TooShort(TooShortError), TooLong(TooLongError) }
/// # impl core::fmt::Display for UserIdError {
/// #     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
/// #         match self { UserIdError::TooShort(e) => e.fmt(f), UserIdError::TooLong(e) => e.fmt(f) }
/// #     }
/// # }
/// # impl core::error::Error for UserIdError {}
/// # impl VouchedError for UserIdError {
/// #     fn as_too_short(&self) -> Option<&TooShortError> {
/// #         match self { UserIdError::TooShort(e) => Some(e), _ => None }
/// #     }
/// #     fn as_too_long(&self) -> Option<&TooLongError> {
/// #         match self { UserIdError::TooLong(e) => Some(e), _ => None }
/// #     }
/// # }
/// #
/// // Function that handles different Vouched error types uniformly.
/// fn process_errors() -> Result<(), Error> {
///     // Any error implementing VouchedError can be converted into Error.
///     let err = UserIdError::TooShort(TooShortError::new(1, 0));
///     Err(Error::from(err))
/// }
///
/// // Inspect the concrete validation error kind.
/// let result = process_errors();
/// assert!(result.is_err());
/// let err = result.unwrap_err();
/// assert!(err.as_too_short().is_some());
/// ```
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct Error(Box<dyn VouchedError>);

#[cfg(feature = "alloc")]
impl Error {
    /// Wraps a boxed generated Vouched error.
    pub fn new(inner: Box<dyn VouchedError>) -> Self {
        Self(inner)
    }

    /// Returns the wrapped generated Vouched error.
    pub fn into_inner(self) -> Box<dyn VouchedError> {
        self.0
    }
}

#[cfg(feature = "alloc")]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "alloc")]
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.0)
    }
}

#[cfg(feature = "alloc")]
impl<E> From<E> for Error
where
    E: VouchedError,
{
    fn from(e: E) -> Self {
        Self(Box::new(e))
    }
}

#[cfg(feature = "alloc")]
impl Deref for Error {
    type Target = dyn VouchedError;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[cfg(feature = "alloc")]
impl AsRef<dyn VouchedError> for Error {
    fn as_ref(&self) -> &(dyn VouchedError + 'static) {
        &*self.0
    }
}

#[cfg(feature = "valuable")]
static ERROR_FIELDS: &[NamedField<'static>] = &[NamedField::new("message")];

#[cfg(feature = "valuable")]
impl Valuable for Error {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        let message = self.to_string();
        let values = [message.as_value()];
        visit.visit_named_fields(&NamedValues::new(ERROR_FIELDS, &values));
    }
}

#[cfg(feature = "valuable")]
impl Structable for Error {
    fn definition(&self) -> StructDef<'_> {
        StructDef::new_static("Error", Fields::Named(ERROR_FIELDS))
    }
}
