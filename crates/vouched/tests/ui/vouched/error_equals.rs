use vouched::Vouched;

#[derive(Vouched)]
#[vouched(error = CustomError, len(1..))]
struct OldErrorSyntax(String);

fn main() {}
