use vouched::Vouched;

// cast(try_from()) with empty type list should be rejected
#[derive(Vouched)]
#[vouched(cast(try_from()))]
struct EmptyCast(i32);

fn main() {}
