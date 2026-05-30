//! Proc-macro implementation for `vouched`.
//!
//! Users normally import [`Vouched`](macro@Vouched) from the facade crate: `use vouched::Vouched;`.

mod vouched;

use proc_macro::TokenStream;

/// Derives validated construction for tuple-struct newtypes.
///
/// `Vouched` supports tuple structs with exactly one field.
/// The generated code validates incoming values and constructs the newtype only when all configured constraints pass.
///
/// # Generated API
///
/// For `struct Name(String)`, the derive generates:
///
/// - `impl TryFrom<String> for Name`.
/// - a generated `NameVouchedError` enum by default.
/// - `Display`, `core::error::Error`, and `vouched::VouchedError` for the generated error enum.
/// - extra integer `TryFrom` implementations requested by `cast(try_from(...))`.
///
/// # Attribute Reference
///
/// Put all options in one or more `#[vouched(...)]` attributes. Each marker can be specified at most once.
///
/// | syntax | effect |
/// | --- | --- |
/// | `len(N..M)` / `len(N..=M)` / `len(N..)` / `len(..M)` / `len(..=M)` | validate string length by Unicode scalar value count |
/// | `chars("abc", '0'..='9', '_')` | validate allowed characters |
/// | `range(N..M)` / `range(N..=M)` / `range(N..)` / `range(..M)` / `range(..=M)` | validate numeric bounds |
/// | `cast(try_from(i64, u32))` | add fallible integer source types |
/// | `error(name = CustomError, vis = pub(crate))` | override the generated error enum name or visibility |
///
/// `error = Name` is not supported.
///
/// # Error Visibility
///
/// The generated error enum uses the derived type's visibility by default.
/// `error(...)` can override the name, the visibility, or both:
///
/// ```ignore
/// # use vouched::Vouched;
/// #[derive(Debug, PartialEq, Eq, Vouched)]
/// #[vouched(error(name = DisplayNameError, vis = pub(crate)), len(1..=32))]
/// pub(crate) struct DisplayName(String);
/// ```
///
/// Rust visibility rules still apply when the generated error type appears in `TryFrom::Error`.
///
/// # Validation Semantics
///
/// Validation returns the first error encountered.
/// When multiple constraints fail, the exact evaluation order is an implementation detail.
/// Implementations may evaluate expensive whole-string validations later to reduce validation cost.
#[proc_macro_derive(Vouched, attributes(vouched))]
pub fn derive_vouched(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    vouched::derive_vouched(&input)
}
