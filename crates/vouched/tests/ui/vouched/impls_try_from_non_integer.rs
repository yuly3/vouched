use vouched::Vouched;

// impls(...) is only supported for integer inner types, not String
#[derive(Vouched)]
#[vouched(len(1..=100), impls(try_from(i64)))]
struct NotInteger(String);

fn main() {}
