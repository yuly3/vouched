use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..=8), impls(try_from(&str)))]
struct BorrowedInner<'a>(&'a str);

fn main() {}
