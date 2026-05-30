use nv::Vouched;

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(len(1..=8))]
pub struct Code(String);

#[derive(Debug, PartialEq, Eq, Vouched)]
#[vouched(range(1..=8))]
pub struct Count(u8);

#[cfg(test)]
mod tests {
    use nv::VouchedError;

    use super::{Code, Count};

    #[test]
    fn renamed_dependency_path_is_used_by_generated_code() {
        assert_eq!(Code::try_from("abc".to_owned()), Ok(Code("abc".to_owned())));
        let err = Count::try_from(9_u8).err();
        assert!(err
            .as_ref()
            .is_some_and(|err| err.as_out_of_range_numeric().is_some()));
    }
}
