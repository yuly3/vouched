use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(0.5..=10.5))]
struct IntegerRange(i32);

fn main() {}
