// crates/ports/src/progress.rs
use count_lines_shared_kernel::Result;

pub trait ProgressSink: Send + Sync {
    fn on_file(&self, path: &std::path::Path) -> Result<()>;
    fn on_complete(&self) -> Result<()>;
}
