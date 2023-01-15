use std::{convert::Infallible, io, os::unix::process::CommandExt, process::Command};

use serde::de;

#[macro_use]
extern crate classic_bitfield;

#[bitfield_enum(as u8)]
pub(crate) enum TestEnum {
    /// first option
    ONE,
    /// second option
    TWO,
    /// third opt
    THREE,
    /// COMBO
    #[repr(0b101)]
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
        assert!(!subject.has_one_and_three());
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

    #[test]
    fn test_with_and_without() {
        let subject = TestEnum::ONE.with(TestEnum::TWO);
        assert_eq!(*subject, 3);
        let subject = subject.without(1);
        assert_eq!(*subject, 2);
        let subject = TestEnum::TWO | TestEnum::THREE;
        assert_eq!(*subject, 6);
        let subject = subject.without(TestEnum::ONE_AND_THREE);
        // note that removing ONE redundantly here is fine.
        assert_eq!(*subject, 2);
    }

    #[test]
    fn test_assigns() {
        let mut subject = TestEnum::ONE;
        subject |= TestEnum::TWO;
        assert_eq!(*subject, 3);
        subject &= TestEnum::ONE_AND_THREE;
        assert_eq!(*subject, 1);
    }

    #[test]
    #[cfg(feature = "serde-as-number")]
    fn test_serialize() {
        let subject = TestEnum::ONE_AND_THREE;
        assert_eq!(serde_json::to_string(&subject).expect("serialize"), "5");
    }
    #[test]
    #[cfg(feature = "serde-as-number")]
    fn test_deserialize() {
        let subject: TestEnum = serde_json::from_str("5").expect("deserialize");
        assert!(subject.has_one_and_three());
    }

    #[test]
    #[cfg(feature = "serde-as-names")]
    fn test_serialize() {
        let subject = TestEnum::ONE_AND_THREE;
        assert_eq!(
            serde_json::to_string(&subject).expect("serialize"),
            r#"["ONE","THREE","ONE_AND_THREE"]"#
        );
    }

    #[test]
    #[cfg(feature = "serde-as-names")]
    fn test_deserialize() {
        let subject: TestEnum = serde_json::from_str(r#"["ONE", "THREE"]"#).expect("deserialize");
        assert!(subject.has_one_and_three());
    }

    #[test]
    fn test_list_names_and_values() {
        assert_eq!(
            TestEnum::variant_names(),
            &["ONE", "TWO", "THREE", "ONE_AND_THREE"]
        );
        assert_eq!(
            TestEnum::variant_values(),
            &[
                TestEnum::ONE,
                TestEnum::TWO,
                TestEnum::THREE,
                TestEnum::ONE_AND_THREE
            ]
        );
        assert_eq!(
            TestEnum::variant_pairs(),
            &[
                ("ONE", TestEnum::ONE),
                ("TWO", TestEnum::TWO),
                ("THREE", TestEnum::THREE),
                ("ONE_AND_THREE", TestEnum::ONE_AND_THREE)
            ]
        );
        let subject = TestEnum::ONE | TestEnum::TWO;
        assert_eq!(subject.names_of_set_variants(), &["ONE", "TWO"]);
    }
}
