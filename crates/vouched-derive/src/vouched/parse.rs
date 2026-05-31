use syn::{
    Attribute, Expr, ExprLit, ExprRange, Ident, Lit, Token, Type, Visibility,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use super::model::{
    CharPattern, DeriveArg, ErrorConfig, ImplConfig, Marker, RangeBound, TryFromSource,
};

impl Parse for ImplConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = Self::default();
        let mut saw_try_from = false;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "try_from" => {
                    if saw_try_from {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "try_from(...) can only be specified once in impls(...)",
                        ));
                    }
                    saw_try_from = true;
                    let content;
                    syn::parenthesized!(content in input);
                    let types = Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
                    config.try_from_sources = types
                        .into_iter()
                        .map(parse_try_from_source)
                        .collect::<syn::Result<Vec<_>>>()?;
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "unknown impl option. Supported: try_from(...)",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if !saw_try_from {
            return Err(syn::Error::new(
                input.span(),
                "impls(...) requires at least one option. Supported: try_from(...)",
            ));
        }

        Ok(config)
    }
}

fn parse_try_from_source(ty: Type) -> syn::Result<TryFromSource> {
    if is_borrowed_str_source(&ty) {
        return Ok(TryFromSource::BorrowedStr(ty));
    }

    if matches!(ty, Type::Reference(_)) {
        return Err(syn::Error::new_spanned(
            ty,
            "unsupported reference type in impls(try_from(...)). Supported reference source: &str",
        ));
    }

    Ok(TryFromSource::Integer(ty))
}

fn is_borrowed_str_source(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => {
            reference.lifetime.is_none()
                && reference.mutability.is_none()
                && is_bare_str_type(reference.elem.as_ref())
        }
        _ => false,
    }
}

fn is_bare_str_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) if type_path.qself.is_none() => type_path.path.is_ident("str"),
        _ => false,
    }
}

impl Parse for ErrorConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = Self::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    if config.ident.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "error name can only be specified once",
                        ));
                    }
                    config.ident = Some(input.parse()?);
                }
                "vis" => {
                    if config.visibility.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "error vis can only be specified once",
                        ));
                    }
                    if !input.peek(Token![pub]) {
                        return Err(syn::Error::new(
                            input.span(),
                            "error vis must start with `pub`, such as `pub` or `pub(crate)`",
                        ));
                    }
                    let visibility: Visibility = input.parse()?;
                    if matches!(visibility, Visibility::Inherited) {
                        return Err(syn::Error::new(
                            input.span(),
                            "error vis must be an explicit Rust visibility",
                        ));
                    }
                    config.visibility = Some(visibility);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "unknown error option. Supported: name = ..., vis = ...",
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if config.ident.is_none() && config.visibility.is_none() {
            return Err(syn::Error::new(
                input.span(),
                "error(...) requires at least one option: name = ... or vis = ...",
            ));
        }

        Ok(config)
    }
}

impl Parse for Marker {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "len" => {
                let content;
                syn::parenthesized!(content in input);
                parse_len_bounds(&content)
            }
            "range" => {
                let content;
                syn::parenthesized!(content in input);
                parse_range_bounds(&content)
            }
            "chars" => {
                let content;
                syn::parenthesized!(content in input);
                parse_chars_marker(&content)
            }
            _ => Err(syn::Error::new_spanned(
                ident,
                "unknown vouched marker. Supported markers: len(<range expr>), range(<range expr>), chars(<char-set>)",
            )),
        }
    }
}

/// Parse len expression like `N..M`, `N..=M`, `N..`, `..M`, `..=M`
fn parse_len_bounds(content: ParseStream) -> syn::Result<Marker> {
    if let Ok(expr) = content.parse::<Expr>() {
        match expr {
            Expr::Range(range) => {
                let lower = match range.start.as_ref() {
                    Some(start) => RangeBound::Inclusive((**start).clone()),
                    None => RangeBound::None,
                };
                let upper = match (&range.limits, range.end.as_ref()) {
                    (syn::RangeLimits::HalfOpen(_), Some(end)) => {
                        RangeBound::Exclusive((**end).clone())
                    }
                    (syn::RangeLimits::Closed(_), Some(end)) => {
                        RangeBound::Inclusive((**end).clone())
                    }
                    (_, None) => RangeBound::None,
                };
                reject_empty_literal_len_range(&range, &lower, &upper)?;
                return Ok(Marker::Len { lower, upper });
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    expr,
                    "len marker requires range syntax like `N..M`, `N..=M`, `N..`, `..M`, or `..=M`",
                ));
            }
        }
    }

    Err(syn::Error::new(
        content.span(),
        "len marker requires range syntax like `N..M`, `N..=M`, `N..`, `..M`, or `..=M`",
    ))
}

fn reject_empty_literal_len_range(
    range: &ExprRange,
    lower: &RangeBound,
    upper: &RangeBound,
) -> syn::Result<()> {
    if is_empty_literal_len_range(lower, upper) {
        return Err(syn::Error::new_spanned(
            range,
            "len marker range is empty; use a non-empty range such as `0..=0` or adjust the bounds",
        ));
    }
    Ok(())
}

fn is_empty_literal_len_range(lower: &RangeBound, upper: &RangeBound) -> bool {
    match literal_len_upper_inclusive(upper) {
        Some(LiteralLenUpper::Empty) => true,
        Some(LiteralLenUpper::Inclusive(max)) => {
            literal_len_lower_inclusive(lower).is_some_and(|min| min > max)
        }
        Some(LiteralLenUpper::Unbounded) | None => false,
    }
}

enum LiteralLenUpper {
    Unbounded,
    Inclusive(usize),
    Empty,
}

fn literal_len_lower_inclusive(lower: &RangeBound) -> Option<usize> {
    match lower {
        RangeBound::None => Some(0),
        RangeBound::Inclusive(expr) => literal_usize(expr),
        RangeBound::Exclusive(expr) => literal_usize(expr).and_then(|value| value.checked_add(1)),
    }
}

fn literal_len_upper_inclusive(upper: &RangeBound) -> Option<LiteralLenUpper> {
    match upper {
        RangeBound::None => Some(LiteralLenUpper::Unbounded),
        RangeBound::Inclusive(expr) => literal_usize(expr).map(LiteralLenUpper::Inclusive),
        RangeBound::Exclusive(expr) => literal_usize(expr).map(|value| {
            value
                .checked_sub(1)
                .map_or(LiteralLenUpper::Empty, LiteralLenUpper::Inclusive)
        }),
    }
}

fn literal_usize(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit.base10_parse::<usize>().ok(),
        _ => None,
    }
}

/// Parse range expression like `N..M`, `N..=M`, `N..`, `..M`, `..=M`
fn parse_range_bounds(content: ParseStream) -> syn::Result<Marker> {
    // Try to parse as a full range expression first
    // syn parses `0..100` as an Expr::Range, so we handle this case
    if let Ok(expr) = content.parse::<Expr>() {
        match expr {
            Expr::Range(range) => {
                let lower = match range.start {
                    Some(start) => RangeBound::Inclusive(*start),
                    None => RangeBound::None,
                };
                let upper = match (range.limits, range.end) {
                    (syn::RangeLimits::HalfOpen(_), Some(end)) => RangeBound::Exclusive(*end),
                    (syn::RangeLimits::Closed(_), Some(end)) => RangeBound::Inclusive(*end),
                    (_, None) => RangeBound::None,
                };
                return Ok(Marker::Range { lower, upper });
            }
            // If we got a non-range expression, it might be a single-bound case
            // But this shouldn't happen with valid range syntax
            _ => {
                return Err(syn::Error::new_spanned(
                    expr,
                    "range marker requires range syntax like `N..M`, `N..=M`, `N..`, `..M`, or `..=M`",
                ));
            }
        }
    }

    Err(syn::Error::new(
        content.span(),
        "range marker requires range syntax like `N..M`, `N..=M`, `N..`, `..M`, or `..=M`",
    ))
}

fn parse_chars_marker(content: ParseStream) -> syn::Result<Marker> {
    let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(content)?;
    let mut patterns = Vec::new();

    for expr in exprs {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }) => {
                for ch in lit.value().chars() {
                    patterns.push(CharPattern::Single(ch));
                }
            }
            Expr::Lit(ExprLit {
                lit: Lit::Char(lit),
                ..
            }) => {
                patterns.push(CharPattern::Single(lit.value()));
            }
            Expr::Range(range) => {
                if !matches!(range.limits, syn::RangeLimits::Closed(_)) {
                    return Err(syn::Error::new_spanned(
                        range,
                        "chars marker only supports inclusive char ranges like `'a'..='z'`",
                    ));
                }

                let Some(start_expr) = range.start.as_ref() else {
                    return Err(syn::Error::new(
                        content.span(),
                        "chars marker range requires both start and end (e.g. `'a'..='z'`)",
                    ));
                };
                let Some(end_expr) = range.end.as_ref() else {
                    return Err(syn::Error::new(
                        content.span(),
                        "chars marker range requires both start and end (e.g. `'a'..='z'`)",
                    ));
                };

                let start = parse_char_expr(start_expr)?;
                let end = parse_char_expr(end_expr)?;
                if start > end {
                    return Err(syn::Error::new_spanned(
                        range,
                        "chars marker range start must be <= end",
                    ));
                }
                patterns.push(CharPattern::InclusiveRange(start, end));
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "chars marker supports string literal, char literal, or inclusive char range like `'a'..='z'`",
                ));
            }
        }
    }

    if patterns.is_empty() {
        return Err(syn::Error::new(
            content.span(),
            "chars marker requires at least one character source",
        ));
    }

    Ok(Marker::Chars { patterns })
}

fn parse_char_expr(expr: &Expr) -> syn::Result<char> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Char(ch), ..
        }) => Ok(ch.value()),
        _ => Err(syn::Error::new_spanned(
            expr,
            "chars marker range boundaries must be char literals",
        )),
    }
}

impl Parse for DeriveArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.fork().parse()?;
        if ident == "impls" {
            input.parse::<Ident>()?; // consume "impls"
            let content;
            syn::parenthesized!(content in input);
            let config: ImplConfig = content.parse()?;
            Ok(Self::Impl(config))
        } else if ident == "error" {
            input.parse::<Ident>()?; // consume "error"
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let error_ident = input.parse::<Ident>()?;
                return Err(syn::Error::new_spanned(
                    error_ident,
                    "`error = Name` is not supported; use `error(name = Name)`",
                ));
            }
            let content;
            syn::parenthesized!(content in input);
            let config: ErrorConfig = content.parse()?;
            Ok(Self::Error(config))
        } else {
            input.parse().map(|m| Self::Marker(Box::new(m)))
        }
    }
}

pub(super) fn parse_vouched_args(
    attrs: &[Attribute],
) -> syn::Result<(Vec<Marker>, Option<ImplConfig>, Option<ErrorConfig>)> {
    let (markers, impl_config, error_config) = attrs
        .iter()
        .filter(|a| a.path().is_ident("vouched"))
        .try_fold((Vec::new(), None, None), fold_vouched_args)?;

    if markers.is_empty() && impl_config.is_none() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Vouched requires at least one validation marker (len/range/chars) or impls(...) in #[vouched(...)]",
        ));
    }

    Ok((markers, impl_config, error_config))
}

fn fold_vouched_args(
    mut acc: (Vec<Marker>, Option<ImplConfig>, Option<ErrorConfig>),
    attr: &Attribute,
) -> syn::Result<(Vec<Marker>, Option<ImplConfig>, Option<ErrorConfig>)> {
    let args = attr.parse_args_with(Punctuated::<DeriveArg, Token![,]>::parse_terminated)?;

    for arg in args {
        match arg {
            DeriveArg::Marker(marker) => {
                reject_duplicate_marker(&acc.0, &marker, attr)?;
                acc.0.push(*marker);
            }
            DeriveArg::Impl(config) => {
                if acc.1.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "impls(...) can only be specified once",
                    ));
                }
                acc.1 = Some(config);
            }
            DeriveArg::Error(config) => {
                if acc.2.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "error(...) can only be specified once",
                    ));
                }
                acc.2 = Some(config);
            }
        }
    }
    Ok(acc)
}

fn reject_duplicate_marker(
    markers: &[Marker],
    marker: &Marker,
    attr: &Attribute,
) -> syn::Result<()> {
    let marker_name = marker_kind_name(marker);
    if markers
        .iter()
        .any(|existing| marker_kind_name(existing) == marker_name)
    {
        return Err(syn::Error::new_spanned(
            attr,
            format!("{marker_name}(...) can only be specified once"),
        ));
    }

    Ok(())
}

fn marker_kind_name(marker: &Marker) -> &'static str {
    match marker {
        Marker::Len { .. } => "len",
        Marker::Range { .. } => "range",
        Marker::Chars { .. } => "chars",
    }
}
