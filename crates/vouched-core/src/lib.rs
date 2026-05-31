//! Core error types used by `vouched` generated code.
//!
//! Most users should depend on the facade crate, `vouched`, which re-exports these types together with the derive macro.
//! This crate exists so generated code can refer to stable validation error types.
//!
//! The important public pieces are:
//!
//! - [`TooShortError`] and [`TooLongError`] for `len(...)` failures.
//! - [`InvalidCharError`] for `chars(...)` failures.
//! - [`OutOfRangeIntegerError`] and [`IntegerValue`] for integer `range(...)` and `impls(try_from(...))` failures.
//! - [`OutOfRangeFloatError`] and [`FloatValue`] for float `range(...)` failures.
//! - [`VouchedError`], implemented by generated error enums.
//! - [`Error`], an allocation-backed erased wrapper available with `alloc`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod str_inner;

pub use error::*;
#[doc(hidden)]
pub use str_inner::VouchedStrInner;
