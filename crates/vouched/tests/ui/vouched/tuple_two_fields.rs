use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..))]
struct NotVouched(i32, i32);

fn main() {}
