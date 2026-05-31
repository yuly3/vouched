use vouched::Vouched;

// impls(try_from()) with empty type list should be rejected
#[derive(Vouched)]
#[vouched(impls(try_from()))]
struct EmptyTryFromImpls(i32);

fn main() {}
