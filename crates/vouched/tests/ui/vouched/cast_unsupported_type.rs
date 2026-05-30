use vouched::Vouched;

// cast(try_from(f64)) - f64 is not supported
#[derive(Vouched)]
#[vouched(range(0..100), cast(try_from(f64)))]
struct UnsupportedSourceType(i32);

fn main() {}
