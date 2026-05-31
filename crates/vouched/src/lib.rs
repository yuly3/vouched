//! Validated tuple-struct newtypes.
//!
//! `vouched` derives checked construction for single-field tuple structs.
//! A value is accepted only after the constraints in `#[vouched(...)]` pass,
//! and the generated `TryFrom` implementations return a structured error when validation fails.
//!
//! Most users only need this facade crate:
//!
//! - [`Vouched`] is the derive macro.
//! - [`VouchedError`] is implemented by generated error enums.
//! - [`Error`] erases different generated error enums behind one type when the `alloc` feature is enabled.
//! - [`TooShortError`], [`TooLongError`], [`InvalidCharError`], [`OutOfRangeIntegerError`], and
//!   [`OutOfRangeFloatError`] are the validation error payloads used by generated error enums.
//!
//! # Quick start
//!
//! ```
//! use vouched::Vouched;
//!
//! #[derive(Debug, PartialEq, Eq, Vouched)]
//! #[vouched(len(1..=64), chars('a'..='z', '0'..='9', '_'))]
//! struct Slug(String);
//! impl Slug {
//!    fn as_str(&self) -> &str {
//!        &self.0
//!    }
//! }
//!
//! let slug = Slug::try_from("hello_123".to_owned())?;
//! assert_eq!(slug.as_str(), "hello_123");
//! # Ok::<(), SlugVouchedError>(())
//! ```
//!
//! # Generated API
//!
//! For a type named `Slug`, `#[derive(Vouched)]` generates:
//!
//! - `impl TryFrom<Inner> for Slug`, where `Inner` is the tuple field type.
//! - additional fallible integer `TryFrom` implementations requested by `impls(try_from(...))`.
//! - a generated error enum named `SlugVouchedError` by default.
//! - `Display`, `core::error::Error`, and [`VouchedError`] for that error enum.
//!
//! The generated error enum uses the derived type's visibility by default.
//! Its name and visibility can be configured with `error(...)`:
//!
//! ```
//! # use vouched::Vouched;
//! #[derive(Debug, PartialEq, Eq, Vouched)]
//! #[vouched(error(name = SlugError, vis = pub(crate)), len(1..=64))]
//! pub(crate) struct Slug(String);
//! ```
//!
//! Rust visibility rules still apply.
//! For example, rustc rejects a public derived type if its generated public `TryFrom` implementation exposes a less
//! visible `TryFrom::Error` type.
//!
//! # Markers
//!
//! | marker | purpose |
//! | --- | --- |
//! | `len(range)` | validates string length by Unicode scalar value count |
//! | `chars(...)` | validates allowed characters from literals and inclusive char ranges |
//! | `range(range)` | validates numeric bounds for supported fixed-width integers, `f32`, and `f64` |
//!
//! `len(...)`, `chars(...)`, and `range(...)` can each be specified at most once.
//! To combine character sets, put all sources in one marker: `chars('a'..='z', '0'..='9', '_')`.
//! `impls(try_from(...))` can also be specified once to add extra fallible integer `TryFrom` implementations before validation.
//!
//! `len(...)` and `chars(...)` use `AsRef<str>` and inspect untrimmed Unicode scalar values. Length is not measured in bytes.
//!
//! `range(...)` type-checks bound expressions against the inner numeric type and generates runtime validation.
//! Float ranges reject an actual `NaN` value as not comparable. Float bound expressions must not evaluate to `NaN`;
//! Rust float comparison rules make a `NaN` bound ineffective. `range(...)` does not prove the range is non-empty.
//!
//! # Validation Semantics
//!
//! Validation returns the first error encountered. When multiple constraints fail,
//! the exact evaluation order is an implementation detail.
//! Implementations may evaluate expensive whole-string validations later to reduce validation cost.
//!
//! # Features
//!
//! | features | crate mode | available |
//! | --- | --- | --- |
//! | default / `std` | `std` | derive macro, core errors, generated errors, erased [`Error`] |
//! | `alloc` without default | `no_std` + `alloc` | derive macro, core errors, generated errors, erased [`Error`] |
//! | no default features | `no_std` | derive macro, core errors, generated errors; no erased [`Error`] |
//! | `valuable` | `no_std` + `alloc` | all `alloc` items plus `valuable::Valuable` implementations for structured error observation |
//!
//! `std` enables `alloc`. `valuable` also enables `alloc`.

#![cfg_attr(not(feature = "std"), no_std)]

// Generated code may refer to this crate by its canonical facade name when the
// macro is expanded from this package's own doctests or source.
extern crate self as vouched;

// Generated code references core types through this facade path,
// including when the facade dependency is renamed by the downstream crate.
#[doc(hidden)]
pub mod __private {
    pub use vouched_core::*;
}

pub use vouched_core::{
    FloatRangeViolation, FloatValue, IntegerValue, InvalidCharError, OutOfRangeFloatError,
    OutOfRangeIntegerError, TooLongError, TooShortError, VouchedError,
};
pub use vouched_derive::Vouched;

#[cfg(feature = "alloc")]
pub use vouched_core::Error;
