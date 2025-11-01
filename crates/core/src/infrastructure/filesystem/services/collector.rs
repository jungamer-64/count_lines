use count_lines_infra::filesystem::PlanFileEnumerator;
use count_lines_ports::filesystem::FileEnumerationPlan;
use count_lines_usecase::CountPaths;

use crate::{
    domain::{config::Config, model::FileEntry},
    error::Result,
};

/// Application service responsible for discovering domain file entries.
pub struct FileEntryCollector;

impl FileEntryCollector {
    pub fn collect(config: &Config) -> Result<Vec<FileEntry>> {
        enumerate(config, EnumerateMode::Default)
    }

    pub fn collect_walk(config: &Config) -> Result<Vec<FileEntry>> {
        enumerate(config, EnumerateMode::WalkOnly)
    }
}

pub fn collect_entries(config: &Config) -> Result<Vec<FileEntry>> {
    FileEntryCollector::collect(config)
}

pub fn collect_walk_entries(config: &Config) -> Result<Vec<FileEntry>> {
    FileEntryCollector::collect_walk(config)
}

enum EnumerateMode {
    Default,
    WalkOnly,
}

fn enumerate(config: &Config, mode: EnumerateMode) -> Result<Vec<FileEntry>> {
    let mut plan = build_plan(config);
    if matches!(mode, EnumerateMode::WalkOnly) {
        plan.files_from = None;
        plan.files_from0 = None;
        plan.use_git = false;
    }

    let enumerator = PlanFileEnumerator::new();
    let usecase = CountPaths::new(&enumerator);
    let output = usecase.run(&plan)?;
    Ok(output.files)
}

fn build_plan(config: &Config) -> FileEnumerationPlan {
    let filters = &config.filters;
    let mut ext_filters: Vec<_> = filters.ext_filters.iter().map(|ext| ext.to_lowercase()).collect();
    ext_filters.sort();
    FileEnumerationPlan {
        roots: config.paths.clone(),
        follow_links: config.follow,
        include_hidden: config.hidden,
        no_default_prune: config.no_default_prune,
        fast_text_detect: config.fast_text_detect,
        include_patterns: patterns_to_strings(&filters.include_patterns),
        exclude_patterns: patterns_to_strings(&filters.exclude_patterns),
        include_paths: patterns_to_strings(&filters.include_paths),
        exclude_paths: patterns_to_strings(&filters.exclude_paths),
        exclude_dirs: patterns_to_strings(&filters.exclude_dirs),
        ext_filters,
        size_range: (filters.size_range.min, filters.size_range.max),
        mtime_since: config.mtime_since,
        mtime_until: config.mtime_until,
        files_from: config.files_from.clone(),
        files_from0: config.files_from0.clone(),
        use_git: config.use_git,
    }
}

fn patterns_to_strings(patterns: &[crate::domain::config::value_objects::GlobPattern]) -> Vec<String> {
    patterns.iter().map(|p| p.pattern().to_string()).collect()
}
