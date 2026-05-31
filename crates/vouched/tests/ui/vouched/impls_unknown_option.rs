use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(0..100), impls(unknown))]
struct UnknownImplOption(i32);

fn main() {}
