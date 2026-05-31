use vouched::Vouched;

struct CustomString(String);

impl From<&str> for CustomString {
    fn from(s: &str) -> Self {
        Self(String::from(s))
    }
}

impl AsRef<str> for CustomString {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Vouched)]
#[vouched(len(1..=4), impls(try_from(&str)))]
struct CustomInner(CustomString);

fn main() {}
