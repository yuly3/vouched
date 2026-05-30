use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

use crate::vouched::model::ErrorKind;

impl ErrorKind {
    pub(crate) fn variant(self, core: &TokenStream2) -> TokenStream2 {
        match self {
            Self::TooShort => quote! { TooShort(#core::TooShortError), },
            Self::TooLong => quote! { TooLong(#core::TooLongError), },
            Self::OutOfRange => quote! { OutOfRange(#core::OutOfRangeNumericError), },
            Self::InvalidChar => quote! { InvalidChar(#core::InvalidCharError), },
        }
    }

    pub(crate) fn display_arm(self, error_ident: &Ident) -> TokenStream2 {
        match self {
            Self::TooShort => {
                quote! { #error_ident::TooShort(e) => ::core::fmt::Display::fmt(e, f), }
            }
            Self::TooLong => {
                quote! { #error_ident::TooLong(e) => ::core::fmt::Display::fmt(e, f), }
            }
            Self::OutOfRange => {
                quote! { #error_ident::OutOfRange(e) => ::core::fmt::Display::fmt(e, f), }
            }
            Self::InvalidChar => {
                quote! { #error_ident::InvalidChar(e) => ::core::fmt::Display::fmt(e, f), }
            }
        }
    }

    pub(crate) fn source_arm(self, error_ident: &Ident) -> TokenStream2 {
        match self {
            Self::TooShort => {
                quote! { #error_ident::TooShort(e) => ::core::option::Option::Some(e), }
            }
            Self::TooLong => {
                quote! { #error_ident::TooLong(e) => ::core::option::Option::Some(e), }
            }
            Self::OutOfRange => {
                quote! { #error_ident::OutOfRange(e) => ::core::option::Option::Some(e), }
            }
            Self::InvalidChar => {
                quote! { #error_ident::InvalidChar(e) => ::core::option::Option::Some(e), }
            }
        }
    }

    pub(crate) fn as_method(self, error_ident: &Ident, core: &TokenStream2) -> TokenStream2 {
        match self {
            Self::TooShort => quote! {
                fn as_too_short(&self) -> ::core::option::Option<&#core::TooShortError> {
                    match self {
                        #error_ident::TooShort(e) => ::core::option::Option::Some(e),
                        _ => ::core::option::Option::None,
                    }
                }
            },
            Self::TooLong => quote! {
                fn as_too_long(&self) -> ::core::option::Option<&#core::TooLongError> {
                    match self {
                        #error_ident::TooLong(e) => ::core::option::Option::Some(e),
                        _ => ::core::option::Option::None,
                    }
                }
            },
            Self::OutOfRange => quote! {
                fn as_out_of_range_numeric(&self) -> ::core::option::Option<&#core::OutOfRangeNumericError> {
                    match self {
                        #error_ident::OutOfRange(e) => ::core::option::Option::Some(e),
                        _ => ::core::option::Option::None,
                    }
                }
            },
            Self::InvalidChar => quote! {
                fn as_invalid_char(&self) -> ::core::option::Option<&#core::InvalidCharError> {
                    match self {
                        #error_ident::InvalidChar(e) => ::core::option::Option::Some(e),
                        _ => ::core::option::Option::None,
                    }
                }
            },
        }
    }
}
