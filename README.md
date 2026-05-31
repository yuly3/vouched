# vouched

`vouched` provides a derive macro for tuple-struct newtypes whose values are vouched for after validation.

## Install

```toml
[dependencies]
vouched = "0.3"
```

## Quick start

```rust
use vouched::Vouched;

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..=64), chars('a'..='z', '0'..='9', '_'), impls(try_from(&str)))]
struct Slug(String);
impl Slug {
    fn as_str(&self) -> &str {
        &self.0
    }
}

let slug = Slug::try_from("hello_123")?;
assert_eq!(slug.as_str(), "hello_123");
```

## Examples

See [crates/vouched/examples](crates/vouched/examples) for more detailed, executable examples.

```sh
cargo run -p vouched --example <name>
```

## Supported Markers

- `len(range)`: validates string length by Unicode scalar value count. Leading and trailing whitespace count.
- `range(range)`: validates numeric bounds for `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `f32`, and `f64`.
- `chars(...)`: validates allowed characters by string literal, char literal, or inclusive char range.

`len(...)`, `range(...)`, and `chars(...)` can each be specified at most once.
To combine character sets, put all sources in one marker, such as `chars('a'..='z', '0'..='9', '_')`.

`range(...)` type-checks the bound expressions against the inner numeric type and generates runtime validation.
Float ranges reject an actual `NaN` value as not comparable.
Float bound expressions must not evaluate to `NaN`; Rust float comparison rules make a `NaN` bound ineffective.
`range(...)` does not guarantee the range is non-empty; for example, a contradictory range remains the user's constraint.

Validation returns the first error encountered. When multiple markers or multiple constraints fail, the exact evaluation order is an implementation detail. The implementation may move expensive whole-string validations later to reduce validation cost.

## Generated API

`#[derive(Vouched)]` generates `TryFrom` impls, a `<TypeName>VouchedError` enum, `Display`, `core::error::Error`, and `vouched::VouchedError`.
Use `impls(try_from(...))` to request additional `TryFrom` implementations before validation.
Supported sources are fallible fixed-width integer conversions and `&str` for string validation newtypes with supported owned string inner types: `String`, `Box<str>`, `Rc<str>`, and `Arc<str>`.
For `impls(try_from(&str))`, the generated implementation validates the borrowed input before constructing the inner value, so invalid inputs do not allocate. Custom string wrapper inner types are not supported by this impl.
The default error enum visibility matches the derived type visibility.
Use `#[vouched(error(name = CustomErrorName, vis = pub(crate)), ...)]` to override the generated error enum name or visibility.
Rust visibility rules still apply: if a public derived type exposes a less-visible error type through `TryFrom::Error`, rustc rejects the generated impl.

## Feature flags

| features | crate mode | available |
| --- | --- | --- |
| default / `std` | `std` | derive macro, core errors, generated errors, erased `Error` |
| `alloc` without default | `no_std` + `alloc` | derive macro, core errors, generated errors, erased `Error` |
| no default features | `no_std` | derive macro, core errors, generated errors; no erased `Error` |
| `valuable` | `no_std` + `alloc` | all `alloc` items plus `valuable::Valuable` impls for structured error observation |

`std` enables `alloc`. `valuable` also enables `alloc`.

## Limitations

- Only tuple structs with exactly one field are supported.
- The default generated error enum name is `<TypeName>VouchedError`; use `error(name = CustomErrorName)` to avoid local name collisions.
- Integer `impls(try_from(...))` supports only fallible fixed-width conversions among `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, and `u128`.
- `impls(try_from(&str))` supports only `len(...)` and/or `chars(...)`, cannot be mixed with integer sources, and requires `String`, `Box<str>`, `Rc<str>`, or `Arc<str>` as the inner type. Custom string wrapper inners and borrowed inners that store the input lifetime are not supported.
- `range(...)` supports fixed-width integers plus `f32` and `f64`; `isize`, `usize`, and custom ordered types are not supported.
- `len(...)` works on `AsRef<str>` values and measures untrimmed Unicode scalar values, not bytes.
- `chars(...)` works on `AsRef<str>` values and validates untrimmed Unicode scalar values.

## Versioning

While `vouched` is in `0.x`, breaking changes to public APIs, generated APIs, or documented behavior require a minor version bump.
Patch releases are limited to bug fixes, documentation updates, and non-breaking additions.

The current MSRV is Rust 1.86. MSRV changes are not shipped in patch releases; while `vouched` is in `0.x`, an MSRV bump is treated as a possibly-breaking compatibility change and shipped in a minor release with release notes.

After `1.0.0`, `vouched` follows SemVer for public APIs and documented behavior.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
