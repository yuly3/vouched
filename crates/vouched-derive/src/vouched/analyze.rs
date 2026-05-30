use syn::{Data, DeriveInput, Fields};

use super::model::{ErrorKind, Marker, RangeBound};

pub(super) fn extract_inner_ty(input: &DeriveInput) -> syn::Result<syn::Type> {
    let data = match &input.data {
        Data::Struct(s) => s,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "Vouched can only be derived for tuple structs (newtype pattern)",
            ));
        }
    };

    match &data.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields
            .unnamed
            .first()
            .map(|field| field.ty.clone())
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    input,
                    "Vouched can only be derived for tuple structs with exactly one field (newtype pattern)",
                )
            }),
        _ => Err(syn::Error::new_spanned(
            input,
            "Vouched can only be derived for tuple structs with exactly one field (newtype pattern)",
        )),
    }
}

pub(super) fn error_kinds_for_markers(
    markers: &[Marker],
    range_error_kind: Option<ErrorKind>,
) -> Vec<ErrorKind> {
    let mut needs_too_short = false;
    let mut needs_too_long = false;
    let mut needs_out_of_range_integer = false;
    let mut needs_out_of_range_float = false;
    let mut needs_invalid_char = false;

    for marker in markers {
        match marker {
            Marker::Len { lower, upper } => {
                if !matches!(lower, RangeBound::None) {
                    needs_too_short = true;
                }
                if !matches!(upper, RangeBound::None) {
                    needs_too_long = true;
                }
            }
            Marker::Range { .. } => match range_error_kind {
                Some(ErrorKind::OutOfRangeInteger) => needs_out_of_range_integer = true,
                Some(ErrorKind::OutOfRangeFloat) => needs_out_of_range_float = true,
                _ => {}
            },
            Marker::Chars { .. } => needs_invalid_char = true,
        }
    }

    let mut kinds = Vec::new();
    if needs_too_short {
        kinds.push(ErrorKind::TooShort);
    }
    if needs_too_long {
        kinds.push(ErrorKind::TooLong);
    }
    if needs_out_of_range_integer {
        kinds.push(ErrorKind::OutOfRangeInteger);
    }
    if needs_out_of_range_float {
        kinds.push(ErrorKind::OutOfRangeFloat);
    }
    if needs_invalid_char {
        kinds.push(ErrorKind::InvalidChar);
    }
    kinds
}
