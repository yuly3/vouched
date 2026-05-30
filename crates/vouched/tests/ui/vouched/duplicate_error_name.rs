use vouched::Vouched;

#[derive(Vouched)]
#[vouched(error(name = FirstError), len(1..))]
#[vouched(error(name = SecondError))]
struct DuplicateErrorName(String);

fn main() {}
