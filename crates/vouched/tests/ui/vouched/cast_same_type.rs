use vouched::Vouched;

// cast(try_from(i32)) when inner type is i32 should be rejected as redundant
#[derive(Vouched)]
#[vouched(range(0..100), cast(try_from(i32)))]
struct SameTypeCast(i32);

fn main() {}
