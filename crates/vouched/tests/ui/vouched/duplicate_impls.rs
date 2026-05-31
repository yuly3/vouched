use vouched::Vouched;

#[derive(Vouched)]
#[vouched(impls(try_from(i64)), impls(try_from(u32)))]
struct DuplicateImpls(i32);

#[derive(Vouched)]
#[vouched(range(0..), impls(try_from(i64), try_from(u32)))]
struct DuplicateTryFromOption(i32);

fn main() {}
