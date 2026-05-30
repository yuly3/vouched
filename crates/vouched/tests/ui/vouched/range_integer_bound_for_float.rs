use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(0..=1))]
struct FloatRange(f64);

fn main() {}
