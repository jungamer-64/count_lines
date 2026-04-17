// crates/cli/src/args.rs
use crate::options::{OutputFormat, SortSpec, WatchOutput};
use crate::parsers::{self, DateTimeArg, SizeArg};
use clap::{Args as ClapArgs, Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "count_lines",
    version,
    about = "ファイル行数/文字数/単語数の集計ツール"
)]
pub struct Args {
    #[command(flatten)]
    pub output: OutputOptions,

    #[command(flatten)]
    pub filter: FilterOptions,

    #[command(flatten)]
    pub scan: ScanOptions,

    #[command(flatten)]
    pub behavior: BehaviorOptions,

    #[command(flatten)]
    pub comparison: ComparisonOptions,

    /// 対象パス
    #[arg(value_hint = ValueHint::AnyPath, help_heading = "走査/入力")]
    pub paths: Vec<PathBuf>,
}

#[derive(ClapArgs, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct OutputOptions {
    /// 出力フォーマット
    #[arg(long, value_enum, default_value = "table", help_heading = "出力")]
    pub format: OutputFormat,

    /// ソートキー（複数可, 例: lines:desc,chars:desc,name）
    #[arg(long, default_value = "lines", help_heading = "出力")]
    pub sort: SortSpec,

    /// CSV/TSV 末尾に TOTAL 行を出力
    #[arg(long, help_heading = "出力")]
    pub total_row: bool,

    /// 改行も文字数に含める
    #[arg(long, help_heading = "出力")]
    pub count_newlines_in_chars: bool,

    /// 進捗表示
    #[arg(long, help_heading = "出力")]
    pub progress: bool,
}

#[derive(ClapArgs, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct FilterOptions {
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub include: Vec<String>,

    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub exclude: Vec<String>,

    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub ext: Vec<String>,

    #[arg(long, help_heading = "フィルタ")]
    pub max_size: Option<SizeArg>,

    #[arg(long, help_heading = "フィルタ")]
    pub min_size: Option<SizeArg>,

    #[arg(long, help_heading = "フィルタ")]
    pub min_lines: Option<usize>,

    #[arg(long, help_heading = "フィルタ")]
    pub max_lines: Option<usize>,

    #[arg(long, help_heading = "フィルタ")]
    pub min_chars: Option<usize>,

    #[arg(long, help_heading = "フィルタ")]
    pub max_chars: Option<usize>,

    #[arg(long, help_heading = "フィルタ")]
    pub words: bool,

    #[arg(long, help_heading = "フィルタ")]
    pub sloc: bool,

    #[arg(long, help_heading = "フィルタ")]
    pub min_words: Option<usize>,

    #[arg(long, help_heading = "フィルタ")]
    pub max_words: Option<usize>,

    #[arg(long, help_heading = "フィルタ")]
    pub mtime_since: Option<DateTimeArg>,

    #[arg(long, help_heading = "フィルタ")]
    pub mtime_until: Option<DateTimeArg>,

    /// 拡張子と言語の紐づけ (例: h=cpp, mylang=sh)
    #[arg(long, value_parser = parsers::parse_key_val, help_heading = "フィルタ")]
    pub map_ext: Vec<(String, String)>,
}

#[derive(ClapArgs, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct ScanOptions {
    #[arg(long, help_heading = "走査/入力")]
    pub hidden: bool,

    #[arg(long, help_heading = "走査/入力")]
    pub follow: bool,

    #[arg(long, help_heading = "走査/入力")]
    pub no_gitignore: bool,

    #[arg(long, value_parser = parsers::parse_usize_1_to_512, help_heading = "走査/入力")]
    pub jobs: Option<usize>,

    #[arg(long = "max-depth", value_parser = parsers::parse_positive_usize, help_heading = "走査/入力")]
    pub max_depth: Option<usize>,

    #[arg(long = "walk-threads", value_parser = parsers::parse_usize_1_to_512, help_heading = "走査/入力")]
    pub walk_threads: Option<usize>,

    #[arg(
        long = "override-include",
        value_delimiter = ',',
        help_heading = "走査/入力"
    )]
    pub override_include: Vec<String>,

    #[arg(
        long = "override-exclude",
        value_delimiter = ',',
        help_heading = "走査/入力"
    )]
    pub override_exclude: Vec<String>,
}

#[derive(ClapArgs, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct BehaviorOptions {
    #[arg(long, help_heading = "動作")]
    pub strict: bool,

    #[arg(short = 'w', long, help_heading = "動作")]
    pub watch: bool,

    #[arg(long = "watch-interval", value_parser = parsers::parse_positive_u64, help_heading = "ウォッチング")]
    pub watch_interval: Option<u64>,

    #[arg(long, value_enum, default_value = "full", help_heading = "動作")]
    pub watch_output: WatchOutput,
}

#[derive(ClapArgs, Debug)]
pub struct ComparisonOptions {
    #[arg(long, num_args = 2, value_names = ["OLD", "NEW"], value_hint = ValueHint::FilePath, help_heading = "比較")]
    pub compare: Option<Vec<PathBuf>>,
}
