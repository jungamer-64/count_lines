// crates/core/src/infrastructure/filesystem/services/collector.rs
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
    let mut plan = FileEnumerationPlan::new();
    plan.roots = config.paths.clone();
    plan.follow_links = config.follow;
    plan.include_hidden = config.hidden;
    plan.no_default_prune = config.no_default_prune;
    plan.fast_text_detect = config.fast_text_detect;
    plan.include_patterns = patterns_to_strings(&filters.include_patterns);
    plan.exclude_patterns = patterns_to_strings(&filters.exclude_patterns);
    plan.include_paths = patterns_to_strings(&filters.include_paths);
    plan.exclude_paths = patterns_to_strings(&filters.exclude_paths);
    plan.exclude_dirs = patterns_to_strings(&filters.exclude_dirs);
    plan.exclude_dirs_only = patterns_to_strings(&filters.exclude_dirs_only);
    plan.ext_filters = ext_filters;
    plan.case_insensitive_dedup = config.case_insensitive_dedup;
    plan.overrides_include = filters.overrides_include.clone();
    plan.overrides_exclude = filters.overrides_exclude.clone();
    plan.force_text_exts = filters.force_text_exts.clone();
    plan.force_binary_exts = filters.force_binary_exts.clone();
    plan.use_ignore_overrides = config.use_ignore_overrides
        || !plan.overrides_include.is_empty()
        || !plan.overrides_exclude.is_empty();
    plan.size_range = (filters.size_range.min, filters.size_range.max);
    plan.mtime_since = config.mtime_since;
    plan.mtime_until = config.mtime_until;
    plan.files_from = config.files_from.clone();
    plan.files_from0 = config.files_from0.clone();
    plan.use_git = config.use_git;
    plan.respect_gitignore = config.respect_gitignore;
    plan.max_depth = config.max_depth;
    plan.threads = config.enumerator_threads;
    plan
}

fn patterns_to_strings(patterns: &[crate::domain::config::value_objects::GlobPattern]) -> Vec<String> {
    patterns.iter().map(|p| p.pattern().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use count_lines_domain::options::OutputFormat;

    use crate::application::queries::config::{
        commands::{ConfigOptions, FilterOptions},
        queries::ConfigQueryService,
    };

    fn base_options() -> ConfigOptions {
        ConfigOptions {
            format: OutputFormat::Json,
            sort_specs: Vec::new(),
            top_n: None,
            by: vec![],
            summary_only: false,
            total_only: false,
            by_limit: None,
            filters: FilterOptions::default(),
            hidden: false,
            follow: false,
            use_git: false,
            respect_gitignore: true,
            case_insensitive_dedup: false,
            max_depth: None,
            enumerator_threads: None,
            jobs: Some(1),
            no_default_prune: false,
            abs_path: false,
            abs_canonical: false,
            trim_root: None,
            words: false,
            sloc: false,
            count_newlines_in_chars: false,
            text_only: false,
            fast_text_detect: false,
            files_from: None,
            files_from0: None,
            paths: vec![std::path::PathBuf::from(".")],
            mtime_since: None,
            mtime_until: None,
            total_row: false,
            progress: false,
            ratio: false,
            output: None,
            strict: false,
            incremental: false,
            cache_dir: None,
            cache_verify: false,
            clear_cache: false,
            watch: false,
            watch_interval: None,
            watch_output: count_lines_domain::options::WatchOutput::Full,
            compare: None,
        }
    }

    #[test]
    fn build_plan_copies_enumerator_controls() {
        let mut options = base_options();
        options.case_insensitive_dedup = true;
        options.respect_gitignore = false;
        options.max_depth = Some(4);
        options.enumerator_threads = Some(6);
        options.filters.overrides_include = vec!["dist/**".into()];
        options.filters.overrides_exclude = vec!["build/**".into()];
        options.filters.force_text_exts = vec!["md".into()];
        options.filters.force_binary_exts = vec!["dat".into()];
        options.filters.exclude_dir_only = vec!["**/tmp/**".into()];

        let config = ConfigQueryService::build(options).expect("config builds");
        let plan = super::build_plan(&config);

        assert!(plan.case_insensitive_dedup);
        assert!(!plan.respect_gitignore);
        assert!(plan.use_ignore_overrides);
        assert_eq!(plan.overrides_include, vec!["dist/**".to_string()]);
        assert_eq!(plan.overrides_exclude, vec!["build/**".to_string()]);
        assert_eq!(plan.force_text_exts, vec!["md".to_string()]);
        assert_eq!(plan.force_binary_exts, vec!["dat".to_string()]);
        assert_eq!(plan.exclude_dirs_only, vec!["**/tmp/**".to_string()]);
        assert_eq!(plan.max_depth, Some(4));
        assert_eq!(plan.threads, Some(6));
    }
}
