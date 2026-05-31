use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..=8), impls(try_from(&str, i64)))]
struct MixedBorrowedStrAndInteger(String);

fn main() {}
