#![allow(
    clippy::unused_unit,
    reason = "derive-generated validation checks currently expand to unit expressions"
)]

use std::{rc::Rc, sync::Arc};

use vouched::{Error, FloatRangeViolation, FloatValue, IntegerValue, Vouched, VouchedError};

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(1..=5), impls(try_from(&str)))]
struct Name(String);

impl Name {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[test]
fn vouched_ok() {
    let v = Name::try_from("Alice");
    assert_eq!(v.as_ref().map(|name| name.as_str()), Ok("Alice"));
}

#[test]
fn vouched_err_first() {
    let v = Name::try_from("");
    let e = v.as_ref().map_err(|err| {
        err.as_too_short()
            .map(|too_short| (too_short.min(), too_short.actual()))
    });
    assert_eq!(e, Err(Some((1, 0))));
}

#[test]
fn vouched_err_second() {
    let v = Name::try_from("TooLong");
    let e = v.as_ref().map_err(|err| {
        err.as_too_long()
            .map(|too_long| (too_long.max(), too_long.actual()))
    });
    assert_eq!(e, Err(Some((5, 7))));
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(1..))]
struct NonEmptyString(String);

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(..=3))]
struct ShortString(String);

fn parse_two(a: String, b: String) -> Result<(NonEmptyString, ShortString), Error> {
    let a = a.try_into()?;
    let b = b.try_into()?;
    Ok((a, b))
}

fn is_out_of_range<T, E: VouchedError>(result: Result<T, E>) -> bool {
    result.is_err_and(|err| err.as_out_of_range_integer().is_some())
}

fn out_of_range_parts<T, E: VouchedError>(
    result: Result<T, E>,
) -> Option<(IntegerValue, Option<IntegerValue>, Option<IntegerValue>)> {
    result.err().and_then(|err| {
        err.as_out_of_range_integer()
            .map(|range| (range.actual(), range.lower_bound(), range.upper_bound()))
    })
}

fn out_of_range_float_parts<T, E: VouchedError>(
    result: Result<T, E>,
) -> Option<(
    FloatValue,
    Option<FloatValue>,
    Option<FloatValue>,
    FloatRangeViolation,
)> {
    result.err().and_then(|err| {
        err.as_out_of_range_float().map(|range| {
            (
                range.actual(),
                range.lower_bound(),
                range.upper_bound(),
                range.violation(),
            )
        })
    })
}

#[test]
fn question_operator_into_erased_error() {
    let err = parse_two("".to_owned(), "abcd".to_owned()).err();
    assert!(err.as_ref().is_some_and(|err| err.as_too_short().is_some()));
    assert!(err.as_ref().is_some_and(|err| err.as_too_long().is_none()));
}

#[test]
fn erased_error_too_long_contains_max() {
    let too_long = parse_two("ok".to_owned(), "abcd".to_owned())
        .err()
        .and_then(|err| {
            err.as_too_long()
                .map(|too_long| (too_long.max(), too_long.actual()))
        });
    assert_eq!(too_long, Some((3, 4)));
}

#[cfg(feature = "valuable")]
#[test]
fn erased_error_is_serialized_as_structured_value() {
    use serde_json::json;
    use valuable_serde::Serializable;

    let value = parse_two("".to_owned(), "ok".to_owned())
        .err()
        .map(|err| serde_json::to_value(Serializable::new(&err)));

    assert!(matches!(
        value,
        Some(Ok(value)) if value == json!({
            "message": "is too short (min 1 chars, got 0)",
        })
    ));
}

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0..100))]
struct RangeHalfOpen(i32);

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0..=100))]
struct RangeClosed(i32);

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0..))]
struct RangeFrom(i32);

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(..100))]
struct RangeTo(i32);

#[test]
fn range_half_open() {
    let r1: Result<RangeHalfOpen, _> = 0.try_into();
    assert!(r1.is_ok());
    let r2: Result<RangeHalfOpen, _> = 99.try_into();
    assert!(r2.is_ok());
    let r3: Result<RangeHalfOpen, _> = 100.try_into();
    assert!(r3.is_err()); // exclusive upper bound
    let r4: Result<RangeHalfOpen, _> = (-1).try_into();
    assert!(r4.is_err());
}

#[test]
fn range_errors_include_actual_and_failed_bound() {
    let upper = out_of_range_parts(RangeHalfOpen::try_from(100_i32));
    assert_eq!(
        upper.map(|(actual, lower_bound, upper_bound)| (
            actual.as_i64(),
            lower_bound,
            upper_bound.and_then(IntegerValue::as_i64)
        )),
        Some((Some(100), None, Some(100)))
    );

    let lower = out_of_range_parts(RangeHalfOpen::try_from(-1_i32));
    assert_eq!(
        lower.map(|(actual, lower_bound, upper_bound)| (
            actual.as_i64(),
            lower_bound.and_then(IntegerValue::as_i64),
            upper_bound
        )),
        Some((Some(-1), Some(0), None))
    );
}

#[test]
fn range_closed() {
    let r1: Result<RangeClosed, _> = 0.try_into();
    assert!(r1.is_ok());
    let r2: Result<RangeClosed, _> = 100.try_into();
    assert!(r2.is_ok()); // inclusive upper bound
    let r3: Result<RangeClosed, _> = 101.try_into();
    assert!(r3.is_err());
    let r4: Result<RangeClosed, _> = (-1).try_into();
    assert!(r4.is_err());
}

#[test]
fn range_from() {
    let r1: Result<RangeFrom, _> = 0.try_into();
    assert!(r1.is_ok());
    let r2: Result<RangeFrom, _> = 1000.try_into();
    assert!(r2.is_ok());
    let r3: Result<RangeFrom, _> = (-1).try_into();
    assert!(r3.is_err());
}

#[test]
fn range_to() {
    let r1: Result<RangeTo, _> = 99.try_into();
    assert!(r1.is_ok());
    let r2: Result<RangeTo, _> = (-1000).try_into();
    assert!(r2.is_ok());
    let r3: Result<RangeTo, _> = 100.try_into();
    assert!(r3.is_err()); // exclusive upper bound
}

// ============================================================================
// Tests for chars(...)
// ============================================================================

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(chars("abc_"), impls(try_from(&str)))]
struct CharsByString(String);
impl CharsByString {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(chars('a'..='z', '0'..='9', '_'), impls(try_from(&str)))]
struct CharsByRange(String);
impl CharsByRange {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(..=3), chars('a'..='z'), impls(try_from(&str)))]
struct LenThenChars(String);

#[test]
fn chars_string_literal_success() {
    let v = CharsByString::try_from("ab_c");
    assert_eq!(v.as_ref().map(|s| s.as_str()), Ok("ab_c"));
}

#[test]
fn chars_string_literal_invalid_char() {
    let v = CharsByString::try_from("ab-c");
    let e = v.as_ref().map_err(|err| {
        err.as_invalid_char()
            .map(|invalid| (invalid.index(), invalid.ch()))
    });
    assert_eq!(e, Err(Some((2, '-'))));
}

#[test]
fn chars_range_literal_success() {
    let v = CharsByRange::try_from("user_123");
    assert_eq!(v.as_ref().map(|s| s.as_str()), Ok("user_123"));
}

#[test]
fn chars_range_literal_invalid_char() {
    let v = CharsByRange::try_from("abc-123");
    let invalid_char = v.as_ref().map_err(|err| {
        err.as_invalid_char()
            .map(|invalid| (invalid.index(), invalid.ch()))
    });
    assert_eq!(invalid_char, Err(Some((3, '-'))));
}

#[test]
fn current_implementation_runs_chars_after_other_validations() {
    let v = LenThenChars::try_from("ab$#");
    let e = v.as_ref().map_err(|err| {
        err.as_too_long()
            .map(|too_long| (too_long.max(), too_long.actual()))
    });
    assert_eq!(e, Err(Some((3, 4))));
}

// ============================================================================
// Tests for impls(try_from(...)) - additional TryFrom impls for integer types
// ============================================================================

/// Vouched with i32 inner type and impls(try_from(i64, u32))
#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0..=100), impls(try_from(i64, u32)))]
struct TryFromRange(i32);
impl TryFromRange {
    fn value(&self) -> i32 {
        self.0
    }
}

#[test]
fn impls_try_from_i64_success() {
    // i64 value within i32 range and within validation range
    let r = TryFromRange::try_from(50_i64);
    assert_eq!(r.as_ref().map(|s| s.value()), Ok(50));
}

#[test]
fn impls_try_from_i64_overflow() {
    // i64 value outside i32 range -> OutOfRange error
    let out_of_range = out_of_range_parts(TryFromRange::try_from(i64::MAX));
    assert_eq!(
        out_of_range.map(|(actual, lower_bound, upper_bound)| (
            actual.as_i64(),
            lower_bound,
            upper_bound
        )),
        Some((Some(i64::MAX), None, None))
    );
}

#[test]
fn impls_try_from_i64_validation_fail() {
    // i64 value within i32 range but outside validation range
    let out_of_range = out_of_range_parts(TryFromRange::try_from(200_i64));
    assert_eq!(
        out_of_range.map(|(actual, lower_bound, upper_bound)| (
            actual.as_i64(),
            lower_bound,
            upper_bound.and_then(IntegerValue::as_i64)
        )),
        Some((Some(200), None, Some(100)))
    );
}

#[test]
fn impls_try_from_i64_negative() {
    // Negative i64 within i32 range but outside validation range (0..=100)
    let r = TryFromRange::try_from(-50_i64);
    assert!(is_out_of_range(r));
}

#[test]
fn impls_try_from_u32_success() {
    // u32 value within i32 range and validation range
    let r = TryFromRange::try_from(75_u32);
    assert_eq!(r.as_ref().map(|s| s.value()), Ok(75));
}

#[test]
fn impls_try_from_u32_overflow() {
    // u32 value outside i32 range (large u32 > i32::MAX) -> OutOfRange error
    let r = TryFromRange::try_from(u32::MAX);
    assert!(is_out_of_range(r));
}

/// Vouched with u8 inner type and extra TryFrom impls from larger types
#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(1..=12), impls(try_from(i32, u32, i64, u64, i128, u128)))]
struct Month(u8);
impl Month {
    fn value(&self) -> u8 {
        self.0
    }
}

#[test]
fn month_from_i32_success() {
    let r = Month::try_from(6_i32);
    assert_eq!(r.as_ref().map(|s| s.value()), Ok(6));
}

#[test]
fn month_from_i32_negative() {
    // Negative i32 -> fails u8 conversion
    let r = Month::try_from(-1_i32);
    assert!(r.is_err());
}

#[test]
fn month_from_i32_overflow() {
    // i32 > 255 -> fails u8 conversion
    let r = Month::try_from(300_i32);
    assert!(r.is_err());
}

#[test]
fn month_from_i32_validation_fail() {
    // i32 within u8 range but outside month range (1..=12)
    let r = Month::try_from(13_i32);
    assert!(r.is_err());
}

#[test]
fn impls_try_from_i128_and_u128_are_supported() {
    let r = Month::try_from(12_i128);
    assert_eq!(r.as_ref().map(|s| s.value()), Ok(12));

    assert_eq!(
        out_of_range_parts(Month::try_from(-1_i128))
            .map(|(actual, _, _)| actual)
            .and_then(IntegerValue::as_i128),
        Some(-1)
    );

    let actual = out_of_range_parts(Month::try_from(u128::MAX)).map(|(actual, _, _)| actual);
    assert_eq!(actual.and_then(IntegerValue::as_u128), Some(u128::MAX));
    assert_eq!(actual.and_then(IntegerValue::as_i128), None);
}

/// Vouched with extra TryFrom impls - OutOfRange is generated for conversion failures
#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0..), impls(try_from(i64)))]
struct IdWithRange(i32);

#[test]
fn id_with_range_try_from_impl_overflow_generates_out_of_range() {
    // This test verifies that OutOfRange error variant is generated
    // because extra TryFrom impls need it for overflow errors
    let r = IdWithRange::try_from(i64::MAX);
    // The error should be OutOfRange for the conversion failure
    assert!(is_out_of_range(r));
}

// ============================================================================
// Tests for impls(try_from(&str)) - borrowed string sources
// ============================================================================

type AliasString = String;

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(1..=4), impls(try_from(&str)))]
struct AliasLabel(AliasString);
impl AliasLabel {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(1..=4), impls(try_from(&str)))]
struct BoxedLabel(Box<str>);
impl BoxedLabel {
    fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(1..=4), impls(try_from(&str)))]
struct RcLabel(Rc<str>);
impl RcLabel {
    fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(1..=4), impls(try_from(&str)))]
struct ArcLabel(Arc<str>);
impl ArcLabel {
    fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[test]
fn impls_try_from_borrowed_str_accepts_common_owned_string_inners() {
    let alias = AliasLabel::try_from("type");
    assert_eq!(alias.as_ref().map(|label| label.as_str()), Ok("type"));

    let boxed = BoxedLabel::try_from("box");
    assert_eq!(boxed.as_ref().map(|label| label.as_str()), Ok("box"));

    let rc = RcLabel::try_from("rc");
    assert_eq!(rc.as_ref().map(|label| label.as_str()), Ok("rc"));

    let arc = ArcLabel::try_from("arc");
    assert_eq!(arc.as_ref().map(|label| label.as_str()), Ok("arc"));

    let too_long = AliasLabel::try_from("alias").err();
    let too_long = too_long
        .as_ref()
        .and_then(|err| err.as_too_long())
        .map(|too_long| (too_long.max(), too_long.actual()));
    assert_eq!(too_long, Some((4, 5)));

    let too_long = BoxedLabel::try_from("boxed").err();
    let too_long = too_long
        .as_ref()
        .and_then(|err| err.as_too_long())
        .map(|too_long| (too_long.max(), too_long.actual()));
    assert_eq!(too_long, Some((4, 5)));

    let too_long = RcLabel::try_from("owned").err();
    let too_long = too_long
        .as_ref()
        .and_then(|err| err.as_too_long())
        .map(|too_long| (too_long.max(), too_long.actual()));
    assert_eq!(too_long, Some((4, 5)));

    let too_long = ArcLabel::try_from("overflow").err();
    let too_long = too_long
        .as_ref()
        .and_then(|err| err.as_too_long())
        .map(|too_long| (too_long.max(), too_long.actual()));
    assert_eq!(too_long, Some((4, 8)));
}

// ============================================================================
// Tests for impls-only Vouched (no validation markers)
// ============================================================================

/// Vouched with only extra TryFrom impls, no validation markers
#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(impls(try_from(i64)))]
struct ImplOnlyId(i32);
impl ImplOnlyId {
    fn value(&self) -> i32 {
        self.0
    }
}

#[test]
fn impls_only_success() {
    let id = ImplOnlyId::try_from(100_i64);
    assert_eq!(id.as_ref().map(|s| s.value()), Ok(100));
}

#[test]
fn impls_only_overflow() {
    let r = ImplOnlyId::try_from(i64::MAX);
    assert!(is_out_of_range(r));
}

#[test]
fn impls_only_inner_try_from_still_works() {
    // TryFrom<i32> (inner type) should still be generated
    let id = ImplOnlyId::try_from(50_i32);
    assert_eq!(id.as_ref().map(|s| s.value()), Ok(50));
}

// ============================================================================
// Boundary and Unicode behavior
// ============================================================================

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(len(2..=2), impls(try_from(&str)))]
struct TwoChars(String);
impl TwoChars {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(chars('界'), impls(try_from(&str)))]
struct OnlyWorldChar(String);
impl OnlyWorldChar {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[test]
fn len_counts_untrimmed_unicode_chars_not_bytes() {
    let valid = TwoChars::try_from("界a");
    assert_eq!(valid.as_ref().map(|s| s.as_str()), Ok("界a"));

    let invalid = TwoChars::try_from("界界a");
    let too_long = invalid.as_ref().map_err(|err| {
        err.as_too_long()
            .map(|too_long| (too_long.max(), too_long.actual()))
    });
    assert_eq!(too_long, Err(Some((2, 3))));
}

#[test]
fn chars_does_not_trim_and_reports_char_index() {
    let valid = OnlyWorldChar::try_from("界界");
    assert_eq!(valid.as_ref().map(|s| s.as_str()), Ok("界界"));

    let err = OnlyWorldChar::try_from("界a").err();
    let invalid_char = err
        .as_ref()
        .and_then(|err| err.as_invalid_char())
        .map(|invalid| (invalid.index(), invalid.ch()));
    assert_eq!(invalid_char, Some((1, 'a')));

    let leading_space_err = OnlyWorldChar::try_from(" 界").err();
    let invalid_char = leading_space_err
        .as_ref()
        .and_then(|err| err.as_invalid_char())
        .map(|invalid| (invalid.index(), invalid.ch()));
    assert_eq!(invalid_char, Some((0, ' ')));
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(range(0..=i64::MAX))]
struct TypedRangeBounds(i64);
impl TypedRangeBounds {
    fn value(&self) -> i64 {
        self.0
    }
}

#[test]
fn range_bounds_accept_unsuffixed_literals_and_same_type_constants() {
    let valid = TypedRangeBounds::try_from(1_i64);
    assert_eq!(valid.as_ref().map(|s| s.value()), Ok(1));
    assert!(TypedRangeBounds::try_from(-1_i64).is_err());
}

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0.0..=1.0))]
struct UnitF32(f32);

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(-1.0..=1.0))]
struct UnitF64(f64);

#[derive(Vouched, Debug, Clone, PartialEq)]
#[vouched(range(0.0..1.0))]
struct HalfOpenF64(f64);

#[test]
fn float_inclusive_ranges_accept_f32_and_f64() {
    assert!(UnitF32::try_from(0.5_f32).is_ok());
    assert!(UnitF32::try_from(1.0_f32).is_ok());
    assert!(UnitF64::try_from(-1.0_f64).is_ok());
    assert!(UnitF64::try_from(1.0_f64).is_ok());
}

#[test]
fn float_half_open_upper_bound_is_exclusive() {
    assert!(HalfOpenF64::try_from(0.5_f64).is_ok());

    let out_of_range = out_of_range_float_parts(HalfOpenF64::try_from(1.0_f64));
    assert_eq!(
        out_of_range.map(|(actual, lower_bound, upper_bound, violation)| (
            actual.as_f64().map(f64::to_bits),
            lower_bound,
            upper_bound.and_then(FloatValue::as_f64).map(f64::to_bits),
            violation,
        )),
        Some((
            Some(1.0_f64.to_bits()),
            None,
            Some(1.0_f64.to_bits()),
            FloatRangeViolation::AboveUpperBound,
        ))
    );
}

#[test]
fn float_range_errors_include_actual_failed_bound_and_violation() {
    let lower = out_of_range_float_parts(UnitF32::try_from(-0.5_f32));
    assert_eq!(
        lower.map(|(actual, lower_bound, upper_bound, violation)| (
            actual.as_f32().map(f32::to_bits),
            lower_bound.and_then(FloatValue::as_f32).map(f32::to_bits),
            upper_bound,
            violation,
        )),
        Some((
            Some((-0.5_f32).to_bits()),
            Some(0.0_f32.to_bits()),
            None,
            FloatRangeViolation::BelowLowerBound,
        ))
    );

    let upper = out_of_range_float_parts(UnitF64::try_from(2.0_f64));
    assert_eq!(
        upper.map(|(actual, lower_bound, upper_bound, violation)| (
            actual.as_f64().map(f64::to_bits),
            lower_bound,
            upper_bound.and_then(FloatValue::as_f64).map(f64::to_bits),
            violation,
        )),
        Some((
            Some(2.0_f64.to_bits()),
            None,
            Some(1.0_f64.to_bits()),
            FloatRangeViolation::AboveUpperBound,
        ))
    );
}

#[test]
fn float_range_rejects_actual_nan_as_not_comparable() {
    let nan = f64::from_bits(0x7ff8_0000_0000_0001);
    let err = UnitF64::try_from(nan).err();
    let out_of_range = err.as_ref().and_then(|err| err.as_out_of_range_float());

    assert_eq!(
        out_of_range.map(|range| (
            range.actual().as_f64().map(f64::to_bits),
            range.lower_bound(),
            range.upper_bound(),
            range.violation(),
            range.to_string(),
        )),
        Some((
            Some(nan.to_bits()),
            None,
            None,
            FloatRangeViolation::NotComparable,
            "not comparable (value NaN)".to_owned(),
        ))
    );
}

#[cfg(feature = "valuable")]
#[test]
fn out_of_range_float_error_is_serialized_as_structured_value() {
    use serde_json::json;
    use valuable_serde::Serializable;

    let value = UnitF64::try_from(2.0_f64).err().and_then(|err| {
        err.as_out_of_range_float()
            .map(|range| serde_json::to_value(Serializable::new(range)))
    });

    assert!(matches!(
        value,
        Some(Ok(value)) if value == json!({
            "actual": 2.0_f64,
            "lower_bound": null,
            "upper_bound": 1.0_f64,
            "violation": "above_upper_bound",
        })
    ));
}

#[test]
fn float_range_uses_regular_float_comparisons_for_infinities_and_signed_zero() {
    let above = out_of_range_float_parts(UnitF64::try_from(f64::INFINITY));
    assert_eq!(
        above.map(|(_, _, _, violation)| violation),
        Some(FloatRangeViolation::AboveUpperBound)
    );

    let below = out_of_range_float_parts(UnitF64::try_from(f64::NEG_INFINITY));
    assert_eq!(
        below.map(|(_, _, _, violation)| violation),
        Some(FloatRangeViolation::BelowLowerBound)
    );

    assert!(UnitF32::try_from(-0.0_f32).is_ok());
}

#[derive(Vouched, Debug, Clone, PartialEq, Eq)]
#[vouched(error(name = CustomLengthError), len(1..=4), impls(try_from(&str)))]
struct CustomLength(String);

enum CustomLengthVouchedError {}

#[test]
fn custom_error_name_avoids_default_name_collision() {
    assert_eq!(core::mem::size_of::<CustomLengthVouchedError>(), 0);
    assert_eq!(
        CustomLength::try_from("hello"),
        Err(CustomLengthError::TooLong(vouched::TooLongError::new(4, 5)))
    );
}

mod configured_error_visibility {
    use vouched::Vouched;

    #[derive(Vouched, Debug, Clone, PartialEq, Eq)]
    #[vouched(
        error(vis = pub(crate), name = CrateVisibleLengthError),
        len(1..=4),
        impls(try_from(&str))
    )]
    pub(crate) struct CrateVisibleLength(String);
}

#[test]
fn custom_error_visibility_and_name_are_configurable() {
    assert_eq!(
        configured_error_visibility::CrateVisibleLength::try_from("hello"),
        Err(
            configured_error_visibility::CrateVisibleLengthError::TooLong(
                vouched::TooLongError::new(4, 5)
            )
        )
    );
}

#[test]
fn integer_value_accessors_are_lossless() {
    let signed = IntegerValue::from(-1_i8);
    assert_eq!(signed.as_i64(), Some(-1));
    assert_eq!(signed.as_u64(), None);
    assert_eq!(signed.as_i128(), Some(-1));
    assert_eq!(signed.as_u128(), None);

    let unsigned = IntegerValue::from(u128::MAX);
    assert_eq!(unsigned.as_u64(), None);
    assert_eq!(unsigned.as_i128(), None);
    assert_eq!(unsigned.as_u128(), Some(u128::MAX));
}

#[test]
fn float_value_accessors_preserve_original_bits_and_width() {
    let nan_bits = 0x7fc0_0001_u32;
    let nan = f32::from_bits(nan_bits);
    let value = FloatValue::from_f32(nan);
    assert_eq!(value.as_f32().map(f32::to_bits), Some(nan_bits));
    assert_eq!(value.as_f64(), None);

    let f64_bits = 0x7ff8_0000_0000_0001_u64;
    let value = FloatValue::from_f64(f64::from_bits(f64_bits));
    assert_eq!(value.as_f64().map(f64::to_bits), Some(f64_bits));
    assert_eq!(value.as_f32(), None);

    assert_eq!(FloatValue::from_f32(nan), FloatValue::from_f32(nan));
    assert_ne!(
        FloatValue::from_f32(0.0_f32),
        FloatValue::from_f32(-0.0_f32)
    );
    assert_ne!(FloatValue::from_f32(0.0_f32), FloatValue::from_f64(0.0_f64));
}
