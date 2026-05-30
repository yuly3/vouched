use vouched::Vouched;

#[derive(Vouched)]
#[vouched(error(), len(1..))]
struct EmptyErrorConfig(String);

#[derive(Vouched)]
#[vouched(error(name = FirstError, name = SecondError), len(1..))]
struct DuplicateErrorNameOption(String);

#[derive(Vouched)]
#[vouched(error(vis = pub(crate), vis = pub(super)), len(1..))]
struct DuplicateErrorVisOption(String);

#[derive(Vouched)]
#[vouched(error(vis = crate), len(1..))]
struct InvalidErrorVis(String);

#[derive(Vouched)]
#[vouched(error(label = Bad), len(1..))]
struct UnknownErrorOption(String);

fn main() {}
