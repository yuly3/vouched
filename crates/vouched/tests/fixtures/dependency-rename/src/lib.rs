use nv::Vouched;

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..=8), impls(try_from(&str)))]
pub struct Code(String);
impl Code {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(range(1..=8))]
pub struct Count(u8);

#[cfg(test)]
mod tests {
    use nv::VouchedError;

    use super::{Code, Count};

    #[test]
    fn renamed_dependency_path_is_used_by_generated_code() {
        assert_eq!(
            Code::try_from("abc").as_ref().map(|code| code.as_str()),
            Ok("abc")
        );
        let err = Count::try_from(9_u8).err();
        assert!(
            err.as_ref()
                .is_some_and(|err| err.as_out_of_range_integer().is_some())
        );
    }
}
