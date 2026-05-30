use vouched::{Vouched, VouchedError};

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..=64), chars('a'..='z', '0'..='9', '_'))]
struct Slug(String);

impl Slug {
    fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_strings() -> Result<(), SlugVouchedError> {
    assert_eq!(
        Slug::try_from("hello_123".to_owned())?.as_str(),
        "hello_123",
    );
    assert_eq!(
        Slug::try_from("release_2026".to_owned())?.as_str(),
        "release_2026",
    );

    Ok(())
}

fn invalid_strings() {
    assert_eq!(
        Slug::try_from(String::new())
            .err()
            .and_then(|err| err.as_too_short().map(|err| (err.min(), err.actual()))),
        Some((1, 0))
    );

    assert_eq!(
        Slug::try_from("hello-123".to_owned())
            .err()
            .and_then(|err| err.as_invalid_char().map(|err| (err.index(), err.ch()))),
        Some((5, '-'))
    );

    assert_eq!(
        Slug::try_from("a".repeat(65))
            .err()
            .and_then(|err| err.as_too_long().map(|err| (err.max(), err.actual()))),
        Some((64, 65))
    );
}

fn main() -> Result<(), SlugVouchedError> {
    valid_strings()?;
    invalid_strings();

    Ok(())
}
