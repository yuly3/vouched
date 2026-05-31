use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..=8), impls(try_from(&str, &str)))]
struct DuplicateBorrowedStr(String);

fn main() {}
