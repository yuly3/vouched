use vouched::Vouched;

#[derive(Vouched)]
#[vouched(len(..0))]
struct LessThanZeroChars(String);

#[derive(Vouched)]
#[vouched(len(0..0))]
struct AtLeastZeroAndLessThanZeroChars(String);

#[derive(Vouched)]
#[vouched(len(1..1))]
struct AtLeastOneAndLessThanOneChar(String);

#[derive(Vouched)]
#[vouched(len(1..=0))]
struct AtLeastOneAndAtMostZeroChars(String);

fn main() {}
