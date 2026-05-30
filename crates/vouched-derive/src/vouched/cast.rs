use syn::Type;

use super::model::CastConfig;

/// Supported integer types for cast operations
pub(crate) const SUPPORTED_INT_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
];

/// Supported float types for range operations
pub(crate) const SUPPORTED_FLOAT_TYPES: &[&str] = &["f32", "f64"];

/// Supported types for range operations
pub(crate) const SUPPORTED_RANGE_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
];

/// Returns the bit width and signedness of a supported integer type
fn int_type_info(ty: &str) -> Option<(u8, bool)> {
    match ty {
        "i8" => Some((8, true)),
        "i16" => Some((16, true)),
        "i32" => Some((32, true)),
        "i64" => Some((64, true)),
        "i128" => Some((128, true)),
        "u8" => Some((8, false)),
        "u16" => Some((16, false)),
        "u32" => Some((32, false)),
        "u64" => Some((64, false)),
        "u128" => Some((128, false)),
        _ => None,
    }
}

pub(crate) fn is_supported_int_type(ty: &Type) -> bool {
    SUPPORTED_INT_TYPES.contains(&type_to_string(ty).as_str())
}

pub(crate) fn is_supported_float_type(ty: &Type) -> bool {
    SUPPORTED_FLOAT_TYPES.contains(&type_to_string(ty).as_str())
}

pub(crate) fn is_supported_range_type(ty: &Type) -> bool {
    SUPPORTED_RANGE_TYPES.contains(&type_to_string(ty).as_str())
}

/// Returns true if converting from `src` to `dst` is fallible (potentially lossy)
/// - Narrowing (larger bit width to smaller) is fallible
/// - Signed to unsigned is fallible (negative values)
/// - Unsigned to signed of same width is fallible (overflow for large values)
fn is_fallible_int_cast(src: &str, dst: &str) -> Option<bool> {
    let (src_bits, src_signed) = int_type_info(src)?;
    let (dst_bits, dst_signed) = int_type_info(dst)?;

    // Same type is not allowed (would be redundant)
    if src == dst {
        return Some(false);
    }

    // Narrowing is always fallible
    if src_bits > dst_bits {
        return Some(true);
    }

    // Same width, different signedness
    if src_bits == dst_bits {
        // Both directions are fallible at same width
        // u32 -> i32: large u32 values overflow
        // i32 -> u32: negative values fail
        return Some(true);
    }

    // Widening (src_bits < dst_bits)
    if src_signed && !dst_signed {
        // i8 -> u16: fallible (negative values)
        return Some(true);
    }
    if !src_signed && dst_signed {
        // u8 -> i16: infallible (always fits)
        return Some(false);
    }
    // Same signedness, widening: infallible
    // i8 -> i16, u8 -> u16: infallible
    Some(false)
}

/// Validate cast configuration and return the list of valid source types
pub(super) fn validate_cast_config(config: &CastConfig, inner_ty: &Type) -> syn::Result<Vec<Type>> {
    // Check if try_from_types is empty
    if config.try_from_types.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "cast(try_from(...)) requires at least one type. Empty type list is not allowed.",
        ));
    }

    let inner_ty_str = type_to_string(inner_ty);

    // Check if inner type is a supported integer type
    if !SUPPORTED_INT_TYPES.contains(&inner_ty_str.as_str()) {
        return Err(syn::Error::new_spanned(
            inner_ty,
            format!(
                "cast(...) is only supported for integer types: {}",
                SUPPORTED_INT_TYPES.join(", ")
            ),
        ));
    }

    let mut validated_types = Vec::new();

    for src_ty in &config.try_from_types {
        let src_ty_str = type_to_string(src_ty);

        // Check if source type is supported
        if !SUPPORTED_INT_TYPES.contains(&src_ty_str.as_str()) {
            return Err(syn::Error::new_spanned(
                src_ty,
                format!(
                    "unsupported type in cast(try_from(...)). Supported types: {}",
                    SUPPORTED_INT_TYPES.join(", ")
                ),
            ));
        }

        // Check if cast is the same type (redundant)
        if src_ty_str == inner_ty_str {
            return Err(syn::Error::new_spanned(
                src_ty,
                format!(
                    "cast(try_from({src_ty_str})) is redundant: source type is the same as inner type"
                ),
            ));
        }

        // Check if cast is fallible
        match is_fallible_int_cast(&src_ty_str, &inner_ty_str) {
            Some(true) => {
                // Fallible cast - this is what we want
                validated_types.push(src_ty.clone());
            }
            Some(false) => {
                // Infallible cast - reject
                return Err(syn::Error::new_spanned(
                    src_ty,
                    format!(
                        "cast(try_from({src_ty_str})) is not allowed: conversion from {src_ty_str} to {inner_ty_str} is infallible. Use `{inner_ty_str}::from({src_ty_str})` before calling try_into() instead."
                    ),
                ));
            }
            None => {
                // Should not happen if SUPPORTED_INT_TYPES is correct
                return Err(syn::Error::new_spanned(
                    src_ty,
                    "internal error: could not determine cast fallibility",
                ));
            }
        }
    }

    Ok(validated_types)
}

/// Convert a Type to its string representation for comparison
pub(crate) fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(ident) = type_path.path.get_ident() {
                return ident.to_string();
            }
            // Handle paths like std::primitive::i32
            type_path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}
