//! `#[derive(Vouched)]` expansion pipeline.
//!
//! Responsibilities are split by phase to keep feature additions local:
//! - `parse`: parse `#[vouched(...)]` arguments into domain model.
//! - `analyze`: inspect derive target and infer required error kinds.
//! - `impls`: validate `impls(try_from(...))` rules for integer conversions.
//! - `codegen`: generate validation checks and unified error enum impls.
//!
//! Marker extension workflow:
//! 1. Add marker shape to `model::Marker`.
//! 2. Parse syntax in `parse`.
//! 3. Add emitted validation in `codegen::markers`.
//! 4. Update `analyze::error_kinds_for_markers` if it introduces new errors.

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Generics, Ident, Type, Visibility};

mod analyze;
mod codegen;
mod impls;
mod model;
mod parse;
mod types;

use analyze::{error_kinds_for_markers, extract_inner_ty};
use impls::validate_try_from_impls;
use model::ErrorKind;
use parse::parse_vouched_args;
use types::{
    SUPPORTED_RANGE_TYPES, is_supported_float_type, is_supported_int_type, is_supported_range_type,
};

struct DerivePlan {
    markers: Vec<model::Marker>,
    try_from_impl_sources: Vec<Type>,
    inner_ty: Type,
    error_kinds: Vec<ErrorKind>,
    error_ident: Ident,
    error_vis: Visibility,
    core_path: TokenStream2,
}

pub fn derive_vouched(input: &DeriveInput) -> TokenStream {
    let plan = match build_plan(input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    TokenStream::from(expand_derive(input, &plan))
}

fn build_plan(input: &DeriveInput) -> syn::Result<DerivePlan> {
    let (markers, impl_config, configured_error_config) = parse_vouched_args(&input.attrs)?;
    let inner_ty = extract_inner_ty(input)?;
    validate_range_markers(&markers, &inner_ty)?;
    let range_error_kind = range_error_kind_for_type(&inner_ty);

    let try_from_impl_sources = if let Some(config) = impl_config.as_ref() {
        validate_try_from_impls(config, &inner_ty)?
    } else {
        Vec::new()
    };

    let mut error_kinds = error_kinds_for_markers(&markers, range_error_kind);
    if !try_from_impl_sources.is_empty() && !error_kinds.contains(&ErrorKind::OutOfRangeInteger) {
        error_kinds.push(ErrorKind::OutOfRangeInteger);
    }

    let configured_error = configured_error_config.unwrap_or_default();
    let error_ident = configured_error.ident.unwrap_or_else(|| {
        syn::Ident::new(&format!("{}VouchedError", input.ident), input.ident.span())
    });
    let error_vis = configured_error
        .visibility
        .unwrap_or_else(|| input.vis.clone());
    let core_path = resolve_core_path(input.ident.span());

    Ok(DerivePlan {
        markers,
        try_from_impl_sources,
        inner_ty,
        error_kinds,
        error_ident,
        error_vis,
        core_path,
    })
}

fn validate_range_markers(markers: &[model::Marker], inner_ty: &Type) -> syn::Result<()> {
    if markers
        .iter()
        .any(|marker| matches!(marker, model::Marker::Range { .. }))
        && !is_supported_range_type(inner_ty)
    {
        return Err(syn::Error::new_spanned(
            inner_ty,
            format!(
                "range(...) is only supported for integer and float types: {}",
                SUPPORTED_RANGE_TYPES.join(", ")
            ),
        ));
    }
    Ok(())
}

fn range_error_kind_for_type(inner_ty: &Type) -> Option<ErrorKind> {
    if is_supported_int_type(inner_ty) {
        Some(ErrorKind::OutOfRangeInteger)
    } else if is_supported_float_type(inner_ty) {
        Some(ErrorKind::OutOfRangeFloat)
    } else {
        None
    }
}

fn resolve_core_path(span: proc_macro2::Span) -> TokenStream2 {
    match crate_name("vouched") {
        Ok(FoundCrate::Itself) => quote! { ::vouched::__private },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name.replace('-', "_"), span);
            quote! { ::#ident::__private }
        }
        Err(_) => quote! { ::vouched::__private },
    }
}

fn expand_derive(input: &DeriveInput, plan: &DerivePlan) -> TokenStream2 {
    expand_derive_with_generics(&input.ident, &input.generics, plan)
}

fn expand_derive_with_generics(
    name: &Ident,
    generics: &Generics,
    plan: &DerivePlan,
) -> TokenStream2 {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let core = &plan.core_path;

    let error_variants = plan
        .error_kinds
        .iter()
        .map(|k| k.variant(core))
        .collect::<Vec<_>>();

    let error_display_arms = plan
        .error_kinds
        .iter()
        .map(|k| k.display_arm(&plan.error_ident))
        .collect::<Vec<_>>();

    let error_source_arms = plan
        .error_kinds
        .iter()
        .map(|k| k.source_arm(&plan.error_ident))
        .collect::<Vec<_>>();

    let error_as_methods = plan
        .error_kinds
        .iter()
        .map(|k| k.as_method(&plan.error_ident, core))
        .collect::<Vec<_>>();

    // Keep expensive character-set checks at the end to reduce total validation cost.
    let ordered_markers = plan
        .markers
        .iter()
        .filter(|m| !matches!(m, model::Marker::Chars { .. }))
        .chain(
            plan.markers
                .iter()
                .filter(|m| matches!(m, model::Marker::Chars { .. })),
        )
        .collect::<Vec<_>>();

    let checks = ordered_markers
        .iter()
        .map(|marker| marker.check_tokens(&plan.inner_ty, &plan.error_ident, core))
        .collect::<Vec<_>>();

    let extra_try_from_impls = plan.try_from_impl_sources.iter().map(|src_ty| {
        let inner_ty = &plan.inner_ty;
        let error_ident = &plan.error_ident;
        quote! {
            impl #impl_generics ::core::convert::TryFrom<#src_ty> for #name #ty_generics #where_clause {
                type Error = #error_ident;

                fn try_from(src_value: #src_ty) -> ::core::result::Result<Self, Self::Error> {
                    // First, convert from source type to inner type using TryFrom
                    let value: #inner_ty = <#inner_ty as ::core::convert::TryFrom<#src_ty>>::try_from(src_value)
                        .map_err(|_| {
                            #error_ident::OutOfRange(
                                #core::OutOfRangeIntegerError::new(
                                    #core::IntegerValue::from(src_value),
                                ),
                            )
                        })?;

                    // Then apply the same validation checks
                    #(
                        #checks;
                    )*
                    ::core::result::Result::Ok(Self(value))
                }
            }
        }
    });

    let inner_ty = &plan.inner_ty;
    let error_ident = &plan.error_ident;
    let error_vis = &plan.error_vis;

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #error_vis enum #error_ident {
            #(#error_variants)*
        }

        impl ::core::fmt::Display for #error_ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#error_display_arms)*
                }
            }
        }

        impl ::core::error::Error for #error_ident {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                match self {
                    #(#error_source_arms)*
                }
            }
        }

        impl #core::VouchedError for #error_ident {
            #(#error_as_methods)*
        }

        impl #impl_generics ::core::convert::TryFrom<#inner_ty> for #name #ty_generics #where_clause {
            type Error = #error_ident;

            fn try_from(value: #inner_ty) -> ::core::result::Result<Self, Self::Error> {
                #(
                    #checks;
                )*
                ::core::result::Result::Ok(Self(value))
            }
        }

        #(#extra_try_from_impls)*
    }
}
