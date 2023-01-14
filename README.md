# Classic Bitfield
A rust crate which tries to create an experience as close to traditional
bit-flag/bitfield value enum ergonomics with as little boilerplate as possible.

## Features
 - dereferences to an underlying representation
 - [**TODO**](https://github.com/dscottboggs/rust-classic-bitfield/issues/1): the underlying representation is customizable
 - implements classic bitwise manipulation with it's own variants (i.e.
   `READ | WRITE & !EXECUTE`) as well as with the underlying representation
   type (i.e. `READ & !1`)
 - also provides convenient methods for combining (`.with()`) and filtering
   (`.without()`).
 - equality and comparison
 - A nice human-readable `fmt::Debug` implementation
 - Serialization and deserialization with serde into either a numeric
   representation or a list of names (*next planned feature*)

## Installation
Add the crate as a dependency:
~~~console
cargo add classic-bitfield
~~~

## Example

```rust
#[bitfield_enum]
pub(crate) enum TestEnum {
    /// first option
    ONE,
    /// second option
    TWO,
    /// third option
    THREE,
    /// COMBO
    #[repr(0b101)]
    ONE_AND_THREE,
}

fn main() {
    let value = TestEnum::ONE | TestEnum::TWO;
    assert!(value.has_one());
    assert!(!value.has_one_and_three());
    let value = value.without(TestEnum::ONE_AND_THREE);
    assert!(!value.has_one());
    assert!(value.has_two());
}
```

For more examples, take a look at [the tests](`classic-bitfield-test/src/main.rs`).