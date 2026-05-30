use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..), len(..=10))]
struct DuplicateLen(String);

#[derive(Vouched)]
#[vouched(range(0..), range(..=10))]
struct DuplicateRange(i32);

#[derive(Vouched)]
#[vouched(chars('a'..='z'), chars('0'..='9'))]
struct DuplicateChars(String);

fn main() {}
