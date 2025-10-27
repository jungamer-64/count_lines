mod args;
mod parsers;
mod value_enum;

use clap::Parser;

use crate::{
    application::{ConfigOptions, ConfigQueryService, FilterOptions},
    domain::config::Config,
};

fn validate_numeric_args(
    top: Option<usize>,
    by_limit: Option<usize>,
    jobs: Option<usize>,
) -> anyhow::Result<()> {
    if let Some(t) = top
        && t == 0
    {
        anyhow::bail!("--top must be at least 1");
    }
    if let Some(bl) = by_limit
        && bl == 0
    {
        anyhow::bail!("--by-limit must be at least 1");
    }
    if let Some(j) = jobs
        && (j == 0 || j > 512)
    {
        anyhow::bail!("--jobs must be between 1 and 512");
    }
    Ok(())
}

pub use args::Args;

/// Parse CLI arguments and materialise a domain [`Config`].
pub fn load_config() -> anyhow::Result<Config> {
    let args = Args::parse();
    build_config(args)
}

/// Convert parsed CLI arguments into a domain configuration.
pub fn build_config(args: Args) -> anyhow::Result<Config> {
    // Validate numeric arguments
    validate_numeric_args(args.top, args.by_limit, args.jobs)?;

    let filter_options = FilterOptions {
        include: args.include,
        exclude: args.exclude,
        include_path: args.include_path,
        exclude_path: args.exclude_path,
        exclude_dir: args.exclude_dir,
        ext: args.ext,
        min_size: args.min_size.map(|s| s.0),
        max_size: args.max_size.map(|s| s.0),
        min_lines: args.min_lines,
        max_lines: args.max_lines,
        min_chars: args.min_chars,
        max_chars: args.max_chars,
        min_words: args.min_words,
        max_words: args.max_words,
        filter: args.filter,
    };

    let compare_tuple =
        args.compare.and_then(|mut v| if v.len() == 2 { Some((v.remove(0), v.remove(0))) } else { None });

    let options = ConfigOptions {
        format: args.format,
        sort_specs: args.sort.0,
        top_n: args.top,
        by: args.by,
        summary_only: args.summary_only,
        total_only: args.total_only,
        by_limit: args.by_limit,
        filters: filter_options,
        hidden: args.hidden,
        follow: args.follow,
        use_git: args.git,
        jobs: args.jobs,
        no_default_prune: args.no_default_prune,
        abs_path: args.abs_path,
        abs_canonical: args.abs_canonical,
        trim_root: args.trim_root,
        words: args.words,
        count_newlines_in_chars: args.count_newlines_in_chars,
        text_only: args.text_only,
        fast_text_detect: args.fast_text_detect,
        files_from: args.files_from,
        files_from0: args.files_from0,
        paths: args.paths,
        mtime_since: args.mtime_since.map(|d| d.0),
        mtime_until: args.mtime_until.map(|d| d.0),
        total_row: args.total_row,
        progress: args.progress,
        ratio: args.ratio,
        output: args.output,
        strict: args.strict,
        compare: compare_tuple,
    };

    ConfigQueryService::build(options)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn min_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--min-words", "5"]);
        let config = build_config(args).expect("config builds");
        assert!(config.words, "min-words should trigger word counting");
    }

    #[test]
    fn sort_by_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--sort", "words:desc"]);
        let config = build_config(args).expect("config builds");
        assert!(config.words, "sorting by words should trigger word counting");
    }

    #[test]
    fn filter_expression_mentioning_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--filter", "words > 10"]);
        let config = build_config(args).expect("config builds");
        assert!(config.words, "filter expressions referencing words should trigger word counting");
    }

    #[test]
    fn abs_canonical_implies_abs_path() {
        let args = Args::parse_from(["count_lines", "--abs-canonical"]);
        let config = build_config(args).expect("config builds");
        assert!(config.abs_canonical);
        assert!(config.abs_path, "--abs-canonical should imply absolute path formatting");
    }
}
