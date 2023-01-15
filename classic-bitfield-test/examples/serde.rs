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
