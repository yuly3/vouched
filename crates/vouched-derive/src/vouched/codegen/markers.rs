use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, LitChar, Type};

use crate::vouched::model::{CharPattern, Marker, RangeBound};

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
}

fn emit_len_check(
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
            let s: &str = ::core::convert::AsRef::<str>::as_ref(&value);
            let len = s.chars().count();
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
    let lower_check = match lower {
        RangeBound::None => quote! {},
        RangeBound::Inclusive(expr) => {
            let out_of_range = emit_out_of_range_below(core);
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
            let out_of_range = emit_out_of_range_below(core);
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
            let out_of_range = emit_out_of_range_above(core);
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
            let out_of_range = emit_out_of_range_above(core);
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

fn emit_out_of_range_below(core: &TokenStream2) -> TokenStream2 {
    quote! {
        #core::OutOfRangeNumericError::new(
            #core::NumericValue::from(value),
        )
        .with_lower_bound(#core::NumericValue::from(lower))
    }
}

fn emit_out_of_range_above(core: &TokenStream2) -> TokenStream2 {
    quote! {
        #core::OutOfRangeNumericError::new(
            #core::NumericValue::from(value),
        )
        .with_upper_bound(#core::NumericValue::from(upper))
    }
}

fn emit_chars_check(
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
            let s: &str = ::core::convert::AsRef::<str>::as_ref(&value);
            for (index, ch) in s.chars().enumerate() {
                if !matches!(ch, #(#match_patterns)|*) {
                    return ::core::result::Result::Err(
                        #error_ident::InvalidChar(#core::InvalidCharError::new(index, ch)),
                    );
                }
            }
        }
    }
}
