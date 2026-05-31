use vouched::Vouched;

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..=64), chars('a'..='z', '0'..='9', '_'), impls(try_from(&str)))]
struct Slug(String);

#[test]
fn readme_quick_start_sample_runs() {
    let slug = Slug::try_from("hello_123");
    assert_eq!(slug, Ok(Slug("hello_123".to_owned())));
}
