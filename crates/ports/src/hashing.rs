// crates/ports/src/hashing.rs
use std::fmt;

use count_lines_shared_kernel::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct HashValue(pub u128);

impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

pub trait Hasher: Send + Sync {
    fn hash_bytes(&self, data: &[u8]) -> Result<HashValue>;
}
