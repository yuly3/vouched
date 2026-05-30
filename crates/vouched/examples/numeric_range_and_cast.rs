use vouched::{NumericValue, Vouched, VouchedError};

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(range(0..=100), cast(try_from(i64, u32)))]
struct Score(i32);

impl Score {
    fn value(&self) -> i32 {
        self.0
    }
}

fn out_of_range_parts(
    result: Result<Score, ScoreVouchedError>,
) -> Option<(NumericValue, Option<NumericValue>, Option<NumericValue>)> {
    result.err().and_then(|err| {
        err.as_out_of_range_numeric()
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
            lower.and_then(NumericValue::as_i64),
            upper.and_then(NumericValue::as_i64),
        )),
        Some((Some(-1), Some(0), None))
    );

    assert_eq!(
        out_of_range_parts(Score::try_from(101_i32)).map(|(actual, lower, upper)| (
            actual.as_i64(),
            lower.and_then(NumericValue::as_i64),
            upper.and_then(NumericValue::as_i64),
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

fn main() -> Result<(), ScoreVouchedError> {
    valid_numbers()?;
    invalid_numbers();

    Ok(())
}
