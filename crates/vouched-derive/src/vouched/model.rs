use syn::{Expr, Ident, Type, Visibility};

/// Configuration for additional generated trait implementations.
#[derive(Clone, Default)]
pub(super) struct ImplConfig {
    pub(super) try_from_types: Vec<Type>,
}

/// Range boundary specification
#[derive(Clone)]
pub(super) enum RangeBound {
    /// No bound (unbounded)
    None,
    /// Exclusive bound (default for upper)
    Exclusive(Expr),
    /// Inclusive bound (default for lower)
    Inclusive(Expr),
}

#[derive(Clone)]
pub(super) enum CharPattern {
    Single(char),
    InclusiveRange(char, char),
}

#[derive(Clone)]
pub(super) enum Marker {
    Len {
        lower: RangeBound,
        upper: RangeBound,
    },
    Range {
        lower: RangeBound,
        upper: RangeBound,
    },
    Chars {
        patterns: Vec<CharPattern>,
    },
}

#[derive(Clone)]
pub(super) enum DeriveArg {
    Marker(Box<Marker>),
    Impl(ImplConfig),
    Error(ErrorConfig),
}

#[derive(Clone, Default)]
pub(super) struct ErrorConfig {
    pub(super) ident: Option<Ident>,
    pub(super) visibility: Option<Visibility>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum ErrorKind {
    TooShort,
    TooLong,
    OutOfRangeInteger,
    OutOfRangeFloat,
    InvalidChar,
}
