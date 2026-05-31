use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(1..=8), impls(try_from(&str)))]
struct BorrowedStrRange(u8);

fn main() {}
