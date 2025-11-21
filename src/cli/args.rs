// src/cli/args.rs
use std::path::PathBuf;

use clap::{Parser, ValueHint};

use super::args_groups::{
    BehaviorOptions, ComparisonOptions, FilterOptions, OutputOptions, PathOptions, ScanOptions,
};

/// Top-level CLI arguments parsed via clap.
#[derive(Parser, Debug)]
#[command(
    name = "count_lines",
    version = crate::VERSION,
    about = "ファイル行数/文字数/単語数の集計ツール",
    long_about = Some(include_str!("../../usage.txt")),
    group(
        clap::ArgGroup::new("input_source")
            .args(&["paths", "files_from", "files_from0"])
            .multiple(false)
    ),
    group(
        clap::ArgGroup::new("abs_mode")
            .args(&["abs_path", "abs_canonical"])
            .multiple(false)
    )
)]
pub struct Args {
    /// Output-related options
    #[command(flatten)]
    pub output: OutputOptions,

    /// Filter-related options
    #[command(flatten)]
    pub filter: FilterOptions,

    /// Scan/Input-related options
    #[command(flatten)]
    pub scan: ScanOptions,

    /// Path output-related options
    #[command(flatten)]
    pub path: PathOptions,

    /// Behavior-related options
    #[command(flatten)]
    pub behavior: BehaviorOptions,

    /// Comparison-related options
    #[command(flatten)]
    pub comparison: ComparisonOptions,

    /// 対象パス
    #[arg(value_hint = ValueHint::AnyPath, help_heading = "走査/入力")]
    pub paths: Vec<PathBuf>,
}
