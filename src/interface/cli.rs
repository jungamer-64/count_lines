mod args;
mod value_enum;

use crate::domain::config::{Config, ConfigOptions, FilterOptions};
use clap::Parser;

pub use args::Args;

/// Parse CLI arguments and materialise a domain [`Config`].
pub fn load_config() -> anyhow::Result<Config> {
    let args = Args::parse();
    build_config(args)
}

/// Convert parsed CLI arguments into a domain configuration.
pub fn build_config(args: Args) -> anyhow::Result<Config> {
    let args::Args {
        format,
        sort,
        top,
        by,
        summary_only,
        total_only,
        by_limit,
        include,
        exclude,
        include_path,
        exclude_path,
        exclude_dir,
        ext,
        max_size,
        min_size,
        min_lines,
        max_lines,
        min_chars,
        max_chars,
        words,
        min_words,
        max_words,
        text_only,
        fast_text_detect,
        files_from,
        files_from0,
        hidden,
        follow,
        git,
        jobs,
        no_default_prune,
        abs_path,
        abs_canonical,
        trim_root,
        total_row,
        mtime_since,
        mtime_until,
        count_newlines_in_chars,
        progress,
        ratio,
        output,
        strict,
        compare,
        filter,
        paths,
    } = args;

    let filter_options = FilterOptions {
        include,
        exclude,
        include_path,
        exclude_path,
        exclude_dir,
        ext,
        min_size: min_size.map(|s| s.0),
        max_size: max_size.map(|s| s.0),
        min_lines,
        max_lines,
        min_chars,
        max_chars,
        min_words,
        max_words,
        filter,
    };

    let compare_tuple = compare.and_then(|mut v| {
        if v.len() == 2 {
            Some((v.remove(0), v.remove(0)))
        } else {
            None
        }
    });

    let options = ConfigOptions {
        format,
        sort_specs: sort.0,
        top_n: top,
        by,
        summary_only,
        total_only,
        by_limit,
        filters: filter_options,
        hidden,
        follow,
        use_git: git,
        jobs,
        no_default_prune,
        abs_path,
        abs_canonical,
        trim_root,
        words,
        count_newlines_in_chars,
        text_only,
        fast_text_detect,
        files_from,
        files_from0,
        paths,
        mtime_since: mtime_since.map(|d| d.0),
        mtime_until: mtime_until.map(|d| d.0),
        total_row,
        progress,
        ratio,
        output,
        strict,
        compare: compare_tuple,
    };

    Config::from_options(options)
}
