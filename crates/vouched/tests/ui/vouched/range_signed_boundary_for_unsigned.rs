use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(-1..=10))]
struct UnsignedRange(u32);

fn main() {}
