use vouched::Vouched;

// i16 -> i32 is infallible (widening, same signedness), should be rejected
#[derive(Vouched)]
#[vouched(range(0..100), impls(try_from(i16)))]
struct InfallibleTryFromImpl(i32);

fn main() {}
