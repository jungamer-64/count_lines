// crates/ports/src/progress.rs
use count_lines_shared_kernel::Result;

pub trait ProgressSink: Send + Sync {
    fn on_file(&self, path: &std::path::Path) -> Result<()>;
    fn on_complete(&self) -> Result<()>;
}

/// Progress sink that discards all events.
#[derive(Debug, Default)]
pub struct NullProgressSink;

impl ProgressSink for NullProgressSink {
    fn on_file(&self, _path: &std::path::Path) -> Result<()> {
        Ok(())
    }

    fn on_complete(&self) -> Result<()> {
        Ok(())
    }
}
