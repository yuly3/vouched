use vouched::{FloatRangeViolation, IntegerValue, Vouched, VouchedError};

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(range(0..=100), cast(try_from(i64, u32)))]
struct Score(i32);

impl Score {
    fn value(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, PartialEq, Vouched)]
#[vouched(range(0.0..=1.0))]
struct Ratio(f64);

fn out_of_range_parts(
    result: Result<Score, ScoreVouchedError>,
) -> Option<(IntegerValue, Option<IntegerValue>, Option<IntegerValue>)> {
    result.err().and_then(|err| {
        err.as_out_of_range_integer()
            .map(|range| (range.actual(), range.lower_bound(), range.upper_bound()))
    })
}

fn valid_numbers() -> Result<(), ScoreVouchedError> {
    assert_eq!(Score::try_from(42_i32)?.value(), 42);
    assert_eq!(Score::try_from(42_i64)?.value(), 42);
    assert_eq!(Score::try_from(42_u32)?.value(), 42);

    Ok(())
}

fn invalid_numbers() {
    assert_eq!(
        out_of_range_parts(Score::try_from(-1_i32)).map(|(actual, lower, upper)| (
            actual.as_i64(),
            lower.and_then(IntegerValue::as_i64),
            upper.and_then(IntegerValue::as_i64),
        )),
        Some((Some(-1), Some(0), None))
    );

    assert_eq!(
        out_of_range_parts(Score::try_from(101_i32)).map(|(actual, lower, upper)| (
            actual.as_i64(),
            lower.and_then(IntegerValue::as_i64),
            upper.and_then(IntegerValue::as_i64),
        )),
        Some((Some(101), None, Some(100)))
    );

    assert_eq!(
        out_of_range_parts(Score::try_from(i64::MAX)).map(|(actual, lower, upper)| (
            actual.as_i64(),
            lower,
            upper,
        )),
        Some((Some(i64::MAX), None, None))
    );

    assert_eq!(
        out_of_range_parts(Score::try_from(u32::MAX)).map(|(actual, lower, upper)| (
            actual.as_u64(),
            lower,
            upper,
        )),
        Some((Some(u64::from(u32::MAX)), None, None))
    );
}

fn valid_ratios() {
    assert!(Ratio::try_from(0.0_f64).is_ok());
    assert!(Ratio::try_from(0.5_f64).is_ok());
    assert!(Ratio::try_from(1.0_f64).is_ok());
}

fn invalid_ratios() {
    let nan_err = Ratio::try_from(f64::NAN).err();
    let violation = nan_err
        .as_ref()
        .and_then(|err| err.as_out_of_range_float())
        .map(|range| range.violation());
    assert_eq!(violation, Some(FloatRangeViolation::NotComparable));
}

fn main() -> Result<(), ScoreVouchedError> {
    valid_numbers()?;
    invalid_numbers();
    valid_ratios();
    invalid_ratios();

    Ok(())
}
