use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(0..100), impls(try_from(i64, i64)))]
struct DuplicateTryFromSource(i32);

fn main() {}
