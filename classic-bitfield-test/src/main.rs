#[macro_use]
extern crate classic_bitfield_macro;

#[bitfield_enum]
pub(crate) enum TestEnum {
    /// first option
    ONE,
    /// second option
    TWO,
    /// third opt
    THREE,
}

fn main() {
    let test = TestEnum::ONE | TestEnum::TWO;
    assert!(test.has_one());
    assert!(test.has_two());
    assert_eq!(test, 3);
    println!("ok")
}
