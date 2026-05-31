use syn::Type;

/// Supported integer types for range checks and extra `TryFrom` implementations.
pub(crate) const SUPPORTED_INT_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
];

/// Supported float types for range operations.
pub(crate) const SUPPORTED_FLOAT_TYPES: &[&str] = &["f32", "f64"];

/// Supported types for range operations.
pub(crate) const SUPPORTED_RANGE_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
];

pub(crate) fn is_supported_int_type(ty: &Type) -> bool {
    SUPPORTED_INT_TYPES.contains(&type_to_string(ty).as_str())
}

pub(crate) fn is_supported_float_type(ty: &Type) -> bool {
    SUPPORTED_FLOAT_TYPES.contains(&type_to_string(ty).as_str())
}

pub(crate) fn is_supported_range_type(ty: &Type) -> bool {
    SUPPORTED_RANGE_TYPES.contains(&type_to_string(ty).as_str())
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
