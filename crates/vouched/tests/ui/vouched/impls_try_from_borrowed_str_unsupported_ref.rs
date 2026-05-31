use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..=8), impls(try_from(&u8)))]
struct UnsupportedReference(String);

fn main() {}
