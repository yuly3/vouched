use vouched::Vouched;

// impls(try_from(i32)) when inner type is i32 should be rejected as redundant
#[derive(Vouched)]
#[vouched(range(0..100), impls(try_from(i32)))]
struct SameTypeTryFromImpl(i32);

fn main() {}
