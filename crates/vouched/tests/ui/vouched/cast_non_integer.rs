use vouched::Vouched;

// cast(...) is only supported for integer inner types, not String
#[derive(Vouched)]
#[vouched(len(1..=100), cast(try_from(i64)))]
struct NotInteger(String);

fn main() {}
