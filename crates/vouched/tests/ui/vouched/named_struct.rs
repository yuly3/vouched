use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(1..))]
struct NotVouched {
    a: i32,
}

fn main() {}
