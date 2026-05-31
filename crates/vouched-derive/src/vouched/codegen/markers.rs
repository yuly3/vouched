use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, LitChar, Type};

use crate::vouched::{
    model::{CharPattern, Marker, RangeBound},
    types::is_supported_float_type,
};

impl CharPattern {
    fn to_match_tokens(&self) -> TokenStream2 {
        match self {
            Self::Single(ch) => {
                let ch = LitChar::new(*ch, proc_macro2::Span::call_site());
                quote! { #ch }
            }
            Self::InclusiveRange(start, end) => {
                let start = LitChar::new(*start, proc_macro2::Span::call_site());
                let end = LitChar::new(*end, proc_macro2::Span::call_site());
                quote! { #start..=#end }
            }
        }
    }
}

impl Marker {
    pub(crate) fn check_tokens(
        &self,
        inner_ty: &Type,
        error_ident: &Ident,
        core: &TokenStream2,
    ) -> TokenStream2 {
        match self {
            Self::Len { lower, upper } => emit_len_check(lower, upper, error_ident, core),
            Self::Range { lower, upper } => {
                emit_range_check(inner_ty, lower, upper, error_ident, core)
            }
            Self::Chars { patterns } => emit_chars_check(patterns, error_ident, core),
        }
    }

    pub(crate) fn check_str_tokens(
        &self,
        error_ident: &Ident,
        core: &TokenStream2,
    ) -> TokenStream2 {
        match self {
            Self::Len { lower, upper } => {
                emit_len_check_from_str(&quote! { s }, lower, upper, error_ident, core)
            }
            Self::Range { .. } => {
                unreachable!("range markers are rejected for impls(try_from(&str))")
            }
            Self::Chars { patterns } => {
                emit_chars_check_from_str(&quote! { s }, patterns, error_ident, core)
            }
        }
    }
}

fn emit_len_check(
    lower: &RangeBound,
    upper: &RangeBound,
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    emit_len_check_from_str(
        &quote! { ::core::convert::AsRef::<str>::as_ref(&value) },
        lower,
        upper,
        error_ident,
        core,
    )
}

fn emit_len_check_from_str(
    str_expr: &TokenStream2,
    lower: &RangeBound,
    upper: &RangeBound,
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    let lower_check = match lower {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => {
            quote! {
                let min: usize = (#expr);
                if len < min {
                    return ::core::result::Result::Err(
                        #error_ident::TooShort(#core::TooShortError::new(min, len))
                    );
                }
            }
        }
        RangeBound::Exclusive(expr) => {
            quote! {
                let lower_exclusive: usize = (#expr);
                if len <= lower_exclusive {
                    let min = lower_exclusive.saturating_add(1);
                    return ::core::result::Result::Err(
                        #error_ident::TooShort(#core::TooShortError::new(min, len))
                    );
                }
            }
        }
    };

    let upper_check = match upper {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => {
            quote! {
                let max: usize = (#expr);
                if len > max {
                    return ::core::result::Result::Err(
                        #error_ident::TooLong(#core::TooLongError::new(max, len)),
                    );
                }
            }
        }
        RangeBound::Exclusive(expr) => {
            quote! {
                let upper_exclusive: usize = (#expr);
                if len >= upper_exclusive {
                    let max = upper_exclusive.saturating_sub(1);
                    return ::core::result::Result::Err(
                        #error_ident::TooLong(#core::TooLongError::new(max, len)),
                    );
                }
            }
        }
    };

    quote! {
        {
            let vouched_str: &str = #str_expr;
            let len = vouched_str.chars().count();
            #lower_check
            #upper_check
        }
    }
}

fn emit_range_check(
    inner_ty: &Type,
    lower: &RangeBound,
    upper: &RangeBound,
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    if is_supported_float_type(inner_ty) {
        return emit_float_range_check(inner_ty, lower, upper, error_ident, core);
    }

    emit_integer_range_check(inner_ty, lower, upper, error_ident, core)
}

fn emit_integer_range_check(
    inner_ty: &Type,
    lower: &RangeBound,
    upper: &RangeBound,
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    let lower_check = match lower {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => {
            let out_of_range = emit_out_of_range_integer_below(core);
            quote! {
                let lower: #inner_ty = (#expr);
                if value < lower {
                    return ::core::result::Result::Err(
                        #error_ident::OutOfRange(#out_of_range),
                    );
                }
            }
        }
        RangeBound::Exclusive(expr) => {
            let out_of_range = emit_out_of_range_integer_below(core);
            quote! {
                let lower: #inner_ty = (#expr);
                if value <= lower {
                    return ::core::result::Result::Err(
                        #error_ident::OutOfRange(#out_of_range),
                    );
                }
            }
        }
    };

    let upper_check = match upper {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => {
            let out_of_range = emit_out_of_range_integer_above(core);
            quote! {
                let upper: #inner_ty = (#expr);
                if value > upper {
                    return ::core::result::Result::Err(
                        #error_ident::OutOfRange(#out_of_range),
                    );
                }
            }
        }
        RangeBound::Exclusive(expr) => {
            let out_of_range = emit_out_of_range_integer_above(core);
            quote! {
                let upper: #inner_ty = (#expr);
                if value >= upper {
                    return ::core::result::Result::Err(
                        #error_ident::OutOfRange(#out_of_range),
                    );
                }
            }
        }
    };

    quote! {
        {
            #lower_check
            #upper_check
        }
    }
}

fn emit_float_range_check(
    inner_ty: &Type,
    lower: &RangeBound,
    upper: &RangeBound,
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    let nan_check = quote! {
        if value.is_nan() {
            return ::core::result::Result::Err(
                #error_ident::OutOfRange(
                    #core::OutOfRangeFloatError::not_comparable(
                        #core::FloatValue::from(value),
                    ),
                ),
            );
        }
    };

    let lower_check = match lower {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => quote! {
            let lower: #inner_ty = (#expr);
            if value < lower {
                return ::core::result::Result::Err(
                    #error_ident::OutOfRange(
                        #core::OutOfRangeFloatError::below_lower_bound(
                            #core::FloatValue::from(value),
                            #core::FloatValue::from(lower),
                        ),
                    ),
                );
            }
        },
        RangeBound::Exclusive(expr) => quote! {
            let lower: #inner_ty = (#expr);
            if value <= lower {
                return ::core::result::Result::Err(
                    #error_ident::OutOfRange(
                        #core::OutOfRangeFloatError::below_lower_bound(
                            #core::FloatValue::from(value),
                            #core::FloatValue::from(lower),
                        ),
                    ),
                );
            }
        },
    };

    let upper_check = match upper {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => quote! {
            let upper: #inner_ty = (#expr);
            if value > upper {
                return ::core::result::Result::Err(
                    #error_ident::OutOfRange(
                        #core::OutOfRangeFloatError::above_upper_bound(
                            #core::FloatValue::from(value),
                            #core::FloatValue::from(upper),
                        ),
                    ),
                );
            }
        },
        RangeBound::Exclusive(expr) => quote! {
            let upper: #inner_ty = (#expr);
            if value >= upper {
                return ::core::result::Result::Err(
                    #error_ident::OutOfRange(
                        #core::OutOfRangeFloatError::above_upper_bound(
                            #core::FloatValue::from(value),
                            #core::FloatValue::from(upper),
                        ),
                    ),
                );
            }
        },
    };

    quote! {
        {
            #nan_check
            #lower_check
            #upper_check
        }
    }
}

fn emit_out_of_range_integer_below(core: &TokenStream2) -> TokenStream2 {
    quote! {
        #core::OutOfRangeIntegerError::new(
            #core::IntegerValue::from(value),
        )
        .with_lower_bound(#core::IntegerValue::from(lower))
    }
}

fn emit_out_of_range_integer_above(core: &TokenStream2) -> TokenStream2 {
    quote! {
        #core::OutOfRangeIntegerError::new(
            #core::IntegerValue::from(value),
        )
        .with_upper_bound(#core::IntegerValue::from(upper))
    }
}

fn emit_chars_check(
    patterns: &[CharPattern],
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    emit_chars_check_from_str(
        &quote! { ::core::convert::AsRef::<str>::as_ref(&value) },
        patterns,
        error_ident,
        core,
    )
}

fn emit_chars_check_from_str(
    str_expr: &TokenStream2,
    patterns: &[CharPattern],
    error_ident: &Ident,
    core: &TokenStream2,
) -> TokenStream2 {
    let match_patterns = patterns
        .iter()
        .map(CharPattern::to_match_tokens)
        .collect::<Vec<_>>();

    quote! {
        {
            let vouched_str: &str = #str_expr;
            for (index, ch) in vouched_str.chars().enumerate() {
                if !matches!(ch, #(#match_patterns)|*) {
                    return ::core::result::Result::Err(
                        #error_ident::InvalidChar(#core::InvalidCharError::new(index, ch)),
                    );
                }
            }
        }
    }
}
