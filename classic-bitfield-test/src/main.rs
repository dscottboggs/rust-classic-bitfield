use std::{convert::Infallible, io, os::unix::process::CommandExt, process::Command};

#[macro_use]
extern crate classic_bitfield;

#[bitfield_enum]
pub(crate) enum TestEnum {
    /// first option
    ONE,
    /// second option
    TWO,
    /// third opt
    THREE,
    /// COMBO
    #[repr(5)]
    ONE_AND_THREE,
}

fn main() -> io::Result<Infallible> {
    Err(Command::new("cargo").arg("test").exec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combo_variant() {
        let subject = TestEnum::ONE_AND_THREE;
        assert!(subject.has_one());
        assert!(!subject.has_two());
        assert!(subject.has_three());
    }

    #[test]
    fn basic_test() {
        let subject = TestEnum::ONE | TestEnum::TWO;
        assert!(subject.has_one());
        assert!(subject.has_two());
        assert_eq!(subject, 3);
    }

    #[test]
    fn debug_output() {
        let subject = TestEnum::ONE;
        assert_eq!(format!("{subject:?}"), "TestEnum::ONE");
        let subject = subject | TestEnum::TWO;
        assert_eq!(format!("{subject:?}"), "TestEnum::ONE | TestEnum::TWO");
        let subject = subject | TestEnum::THREE;
        assert_eq!(
            format!("{subject:?}"),
            "TestEnum::ONE | TestEnum::TWO | TestEnum::THREE | TestEnum::ONE_AND_THREE"
        );
    }
}
