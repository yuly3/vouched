use vouched::Vouched;

const LOWER: u32 = 0;
const UPPER: u32 = 10;

#[derive(Vouched)]
#[vouched(range(LOWER..=UPPER))]
struct SignedRange(i32);

fn main() {}
