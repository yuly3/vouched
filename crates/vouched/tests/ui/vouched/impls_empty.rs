use vouched::Vouched;

#[derive(Vouched)]
#[vouched(impls())]
struct EmptyImpls(i32);

fn main() {}
