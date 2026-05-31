use vouched::Vouched;

#[derive(Vouched)]
#[vouched(impls(try_from(&str)))]
struct MissingStringValidation(String);

fn main() {}
