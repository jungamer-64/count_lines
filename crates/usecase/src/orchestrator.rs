use count_lines_domain::model::FileEntry;
use count_lines_ports::filesystem::{FileEntryDto as PortFileEntry, FileEnumerationPlan, FileEnumerator};
use count_lines_shared_kernel::{Result, value_objects::FileMeta};

use crate::dto::CountEntriesOutput;

pub struct CountPaths<'a> {
    enumerator: &'a dyn FileEnumerator,
}

impl<'a> CountPaths<'a> {
    pub fn new(enumerator: &'a dyn FileEnumerator) -> Self {
        Self { enumerator }
    }

    pub fn run(&self, plan: &FileEnumerationPlan) -> Result<CountEntriesOutput> {
        let entries = self.enumerate(plan)?;
        Ok(CountEntriesOutput { files: entries })
    }

    fn enumerate(&self, plan: &FileEnumerationPlan) -> Result<Vec<FileEntry>> {
        let ports_entries = self.enumerator.collect(plan)?;
        Ok(ports_entries.into_iter().map(port_to_domain_entry).collect())
    }
}

fn port_to_domain_entry(entry: PortFileEntry) -> FileEntry {
    let meta = FileMeta {
        size: entry.size,
        mtime: entry.mtime,
        is_text: entry.is_text,
        ext: entry.ext,
        name: entry.name,
    };
    FileEntry { path: entry.path, meta }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::Local;

    use super::*;

    #[derive(Default)]
    struct StubEnumerator {
        entries: Mutex<Vec<PortFileEntry>>,
    }

    impl StubEnumerator {
        fn with_entry(path: &str) -> Self {
            let dto = PortFileEntry {
                path: path.into(),
                is_text: true,
                size: 42,
                ext: "txt".into(),
                name: "sample".into(),
                mtime: Some(Local::now()),
            };
            Self { entries: Mutex::new(vec![dto]) }
        }
    }

    impl FileEnumerator for StubEnumerator {
        fn collect(&self, _plan: &FileEnumerationPlan) -> Result<Vec<PortFileEntry>> {
            Ok(self.entries.lock().unwrap().clone())
        }
    }

    #[test]
    fn run_returns_entries() {
        let stub = StubEnumerator::with_entry("sample.txt");
        let usecase = CountPaths::new(&stub);
        let plan = FileEnumerationPlan {
            roots: vec![],
            follow_links: false,
            include_hidden: false,
            no_default_prune: false,
            fast_text_detect: true,
            include_patterns: vec![],
            exclude_patterns: vec![],
            include_paths: vec![],
            exclude_paths: vec![],
            exclude_dirs: vec![],
            ext_filters: vec![],
            size_range: (None, None),
            mtime_since: None,
            mtime_until: None,
            files_from: None,
            files_from0: None,
            use_git: false,
        };
        let output = usecase.run(&plan).expect("run succeeds");
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, std::path::PathBuf::from("sample.txt"));
        assert_eq!(output.files[0].meta.name, "sample");
    }
}
