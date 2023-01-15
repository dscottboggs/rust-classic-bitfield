# Classic Bitfield
A rust crate which tries to create an experience as close to traditional
bit-flag/bitfield value enum ergonomics with as little boilerplate as possible.

## Features
 - dereferences to an underlying representation
 - the underlying representation is customizable (i.e. the type is representable
   as various sizes; e.g. `u8`, `i128`, etc.)
 - implements classic bitwise manipulation with it's own variants (i.e.
   `READ | WRITE & !EXECUTE`) as well as with the underlying representation
   type (i.e. `READ & !1`)
 - also provides convenient methods for combining (`.with()`) and filtering
   (`.without()`).
 - equality and comparison
 - A nice human-readable `fmt::Debug` implementation
 - Serialization and deserialization with serde into either a numeric
   representation or a list of names

## Installation
Add the crate as a dependency:
~~~console
cargo add classic-bitfield
~~~

## Example

```rust
#[bitfield_enum(as u8)]
pub(crate) enum Permissions {
    /// Permission to run executables or list directories
    EXECUTE,
    /// Permssion to write to the file
    WRITE,
    /// Permission to read to the file
    READ,
    /// COMBO
    #[repr(0o6)]
    READ_AND_WRITE,
}

fn main() {
    let value = Permissions::all_set();
    assert!(value.has_execute());
    assert!(!value.has_read_and_write());
    let value = value.without(Permissions::EXECUTE);
    assert!(!value.has_execute());
    assert!(value.has_write());
}
```
With `--features=serde` (requires `serde`; example requires `serde_json` and
`serde`'s `"derive"` feature)

```rust
use std::io::stdout;

use classic_bitfield::bitfield_enum;
use serde::{Deserialize, Serialize};

#[bitfield_enum(as u8)]
pub(crate) enum Permissions {
    /// Permission to run executables or list directories
    EXECUTE,
    /// Permssion to write to the file
    WRITE,
    /// Permission to read to the file
    READ,
    /// COMBO
    #[repr(0o6)]
    READ_AND_WRITE,
}

use permissions_serde::numeric_representation;

#[derive(Serialize, Deserialize)]
struct FileMetadata {
    #[serde(with = "numeric_representation")]
    permissions: Permissions,
}

fn main() {
    let stdout = stdout().lock();
    let example = FileMetadata {
        permissions: Permissions::READ_AND_WRITE,
    };
    serde_json::to_writer_pretty(stdout, &example).unwrap();
    println!();
}
```
The output from the above example is:
```json
{
  "permissions": 6
}
```

To get an idea of what features will be available on your generated type, take
a look at [the tests](`classic-bitfield-test/src/main.rs`).

## Limitations

 - Your linter will rightly complain if you try to name the enum variants in
`CamelCase` like you would a regular enum. This is appropriate and desirable
&mdash; bitfield variants are *not* distinct types, they are constants, and
styling them this way ensures that fact is kept in mind.
 - currently the types which are representable are limited to the standard
 library signed and unsigned integer types. Implementations for other types
 will be considered should use-cases arise.