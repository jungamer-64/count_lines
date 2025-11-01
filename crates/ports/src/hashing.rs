use count_lines_shared_kernel::Result;

#[derive(Debug, Clone, Copy)]
pub struct HashValue(pub u128);

pub trait Hasher: Send + Sync {
    fn hash_bytes(&self, data: &[u8]) -> Result<HashValue>;
}
