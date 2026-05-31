use syn::Type;

use super::{
    model::ImplConfig,
    types::{SUPPORTED_INT_TYPES, type_to_string},
};

/// Returns the bit width and signedness of a supported integer type.
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

/// Returns true if converting from `src` to `dst` is fallible.
fn is_fallible_int_conversion(src: &str, dst: &str) -> Option<bool> {
    let (src_bits, src_signed) = int_type_info(src)?;
    let (dst_bits, dst_signed) = int_type_info(dst)?;

    if src == dst {
        return Some(false);
    }

    if src_bits > dst_bits {
        return Some(true);
    }

    if src_bits == dst_bits {
        return Some(true);
    }

    if src_signed && !dst_signed {
        return Some(true);
    }
    if !src_signed && dst_signed {
        return Some(false);
    }

    Some(false)
}

/// Validate `impls(try_from(...))` and return the valid source types.
pub(super) fn validate_try_from_impls(
    config: &ImplConfig,
    inner_ty: &Type,
) -> syn::Result<Vec<Type>> {
    if config.try_from_types.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "impls(try_from(...)) requires at least one type. Empty type list is not allowed.",
        ));
    }

    let inner_ty_str = type_to_string(inner_ty);

    if !SUPPORTED_INT_TYPES.contains(&inner_ty_str.as_str()) {
        return Err(syn::Error::new_spanned(
            inner_ty,
            format!(
                "impls(try_from(...)) is only supported for integer inner types: {}",
                SUPPORTED_INT_TYPES.join(", ")
            ),
        ));
    }

    let mut validated_types = Vec::new();
    let mut seen_sources = Vec::new();

    for src_ty in &config.try_from_types {
        let src_ty_str = type_to_string(src_ty);

        if !SUPPORTED_INT_TYPES.contains(&src_ty_str.as_str()) {
            return Err(syn::Error::new_spanned(
                src_ty,
                format!(
                    "unsupported type in impls(try_from(...)). Supported types: {}",
                    SUPPORTED_INT_TYPES.join(", ")
                ),
            ));
        }

        if seen_sources.iter().any(|seen| seen == &src_ty_str) {
            return Err(syn::Error::new_spanned(
                src_ty,
                format!("duplicate source type in impls(try_from(...)): {src_ty_str}"),
            ));
        }

        if src_ty_str == inner_ty_str {
            return Err(syn::Error::new_spanned(
                src_ty,
                format!(
                    "impls(try_from({src_ty_str})) is redundant: source type is the same as inner type"
                ),
            ));
        }

        match is_fallible_int_conversion(&src_ty_str, &inner_ty_str) {
            Some(true) => {
                seen_sources.push(src_ty_str);
                validated_types.push(src_ty.clone());
            }
            Some(false) => {
                return Err(syn::Error::new_spanned(
                    src_ty,
                    format!(
                        "impls(try_from({src_ty_str})) is not allowed: conversion from {src_ty_str} to {inner_ty_str} is infallible. Use `{inner_ty_str}::from({src_ty_str})` before calling try_into() instead."
                    ),
                ));
            }
            None => {
                return Err(syn::Error::new_spanned(
                    src_ty,
                    "internal error: could not determine conversion fallibility",
                ));
            }
        }
    }

    Ok(validated_types)
}
