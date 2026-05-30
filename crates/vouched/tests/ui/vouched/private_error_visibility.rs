use vouched::Vouched;

mod inner {
    use super::Vouched;

    #[derive(Vouched)]
    #[vouched(len(1..))]
    struct PrivateName(String);
}

fn main() {
    let _: Option<inner::PrivateNameVouchedError> = None;
}
