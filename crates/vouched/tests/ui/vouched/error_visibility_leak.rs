use vouched::Vouched;

#[derive(Vouched)]
#[vouched(error(vis = pub(crate), name = PublicNameError), len(1..))]
pub struct PublicName(String);

fn main() {}
