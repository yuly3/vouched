use vouched::{Error, Vouched};

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..), chars('a'..='z', '0'..='9', '_'))]
struct Username(String);

impl Username {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..=32))]
struct DisplayName(String);

impl DisplayName {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AccountProfile {
    username: Username,
    display_name: DisplayName,
}

impl AccountProfile {
    fn try_new(username: &str, display_name: &str) -> Result<Self, Error> {
        Ok(Self {
            username: Username::try_from(username.to_owned())?,
            display_name: DisplayName::try_from(display_name.to_owned())?,
        })
    }
    fn username(&self) -> &Username {
        &self.username
    }
    fn display_name(&self) -> &DisplayName {
        &self.display_name
    }
}

fn too_short_parts(err: Option<&Error>) -> Option<(usize, usize)> {
    err.and_then(|err| err.as_too_short())
        .map(|err| (err.min(), err.actual()))
}

fn too_long_parts(err: Option<&Error>) -> Option<(usize, usize)> {
    err.and_then(|err| err.as_too_long())
        .map(|err| (err.max(), err.actual()))
}

fn invalid_char_parts(err: Option<&Error>) -> Option<(usize, char)> {
    err.and_then(|err| err.as_invalid_char())
        .map(|err| (err.index(), err.ch()))
}

fn accepted_profile() -> Result<(), Error> {
    let account_profile = AccountProfile::try_new("alice_123", "Alice")?;
    assert_eq!(account_profile.username().as_str(), "alice_123");
    assert_eq!(account_profile.display_name().as_str(), "Alice");
    Ok(())
}

fn rejected_profile() {
    let empty_username = AccountProfile::try_new("", "Alice").err();
    assert_eq!(too_short_parts(empty_username.as_ref()), Some((1, 0)));

    let bad_username = AccountProfile::try_new("alice-123", "Alice").err();
    assert_eq!(invalid_char_parts(bad_username.as_ref()), Some((5, '-')));

    let long_display_name = AccountProfile::try_new("alice_123", "A".repeat(33).as_str()).err();
    assert_eq!(too_long_parts(long_display_name.as_ref()), Some((32, 33)));
}

fn main() -> Result<(), Error> {
    accepted_profile()?;
    rejected_profile();

    Ok(())
}
