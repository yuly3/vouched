use vouched::Vouched;

#[derive(Vouched)]
#[vouched(range(0..=10))]
struct PointerSized(usize);

fn main() {}
