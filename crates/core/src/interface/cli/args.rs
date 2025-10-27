// crates/core/src/interface/cli/args.rs
use crate::domain::grouping::ByMode;
use crate::domain::options::{OutputFormat, SortSpec};
use crate::foundation::util::{DateTimeArg, SizeArg};
use clap::{Parser, ValueHint};
use std::path::PathBuf;

/// Top-level CLI arguments parsed via clap.
#[derive(Parser, Debug)]
#[command(
    name = "count_lines",
    version = crate::VERSION,
    about = "ファイル行数/文字数/単語数の集計ツール",
    long_about = Some(include_str!("../../../../../usage.txt")),
    // ファイル入力ソースは排他的（paths / files_from / files_from0）
    group(
        clap::ArgGroup::new("input_source")
            .args(&["paths", "files_from", "files_from0"])
            .multiple(false)
    ),
    // 絶対パス出力モードは排他的（abs_path / abs_canonical）
    group(
        clap::ArgGroup::new("abs_mode")
            .args(&["abs_path", "abs_canonical"])
            .multiple(false)
    )
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// 出力フォーマット
    #[arg(long, value_enum, default_value = "table", help_heading = "出力")]
    pub format: OutputFormat,

    /// ソートキー（複数可, 例: lines:desc,chars:desc,name）
    #[arg(long, default_value = "lines:desc", help_heading = "出力")]
    pub sort: SortSpec,

    /// 上位N件のみ表示（一覧）
    #[arg(long, help_heading = "出力")]
    pub top: Option<usize>,

    /// サマリ軸 (ext, dir, dir=N, mtime[:day|week|month]) — 複数可
    #[arg(long, help_heading = "出力")]
    pub by: Vec<ByMode>,

    /// サマリのみ表示（一覧は出力しないが By 集計は出す）
    #[arg(long, conflicts_with = "total_only", help_heading = "出力")]
    pub summary_only: bool,

    /// 合計のみ表示（一覧と By 集計は出さない）
    #[arg(long, help_heading = "出力")]
    pub total_only: bool,

    /// 集計テーブルの上位N件のみ表示
    #[arg(long, requires = "by", help_heading = "出力")]
    pub by_limit: Option<usize>,

    /// 含めるファイル名パターン（カンマ区切り/複数指定可）
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub include: Vec<String>,

    /// 除外するファイル名パターン（カンマ区切り/複数指定可）
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub exclude: Vec<String>,

    /// 含めるパスパターン（カンマ区切り/複数指定可）
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub include_path: Vec<String>,

    /// 除外するパスパターン（カンマ区切り/複数指定可）
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub exclude_path: Vec<String>,

    /// 除外ディレクトリパターン（カンマ区切り/複数指定可）
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub exclude_dir: Vec<String>,

    /// 拡張子フィルタ (カンマ区切り)
    #[arg(long, help_heading = "フィルタ")]
    pub ext: Option<String>,

    /// 最大ファイルサイズ (例: 10K, 5MiB)
    #[arg(long, help_heading = "フィルタ")]
    pub max_size: Option<SizeArg>,

    /// 最小ファイルサイズ (例: 10K, 5MiB)
    #[arg(long, help_heading = "フィルタ")]
    pub min_size: Option<SizeArg>,

    /// 最小行数
    #[arg(long, help_heading = "フィルタ")]
    pub min_lines: Option<usize>,

    /// 最大行数
    #[arg(long, help_heading = "フィルタ")]
    pub max_lines: Option<usize>,

    /// 最小文字数
    #[arg(long, help_heading = "フィルタ")]
    pub min_chars: Option<usize>,

    /// 最大文字数
    #[arg(long, help_heading = "フィルタ")]
    pub max_chars: Option<usize>,

    /// 単語数も計測
    #[arg(long, help_heading = "フィルタ")]
    pub words: bool,

    /// 最小単語数（指定時は --words を暗黙に有効化する実装推奨）
    #[arg(long, help_heading = "フィルタ")]
    pub min_words: Option<usize>,

    /// 最大単語数（指定時は --words を暗黙に有効化する実装推奨）
    #[arg(long, help_heading = "フィルタ")]
    pub max_words: Option<usize>,

    /// テキストファイルのみ
    #[arg(long, help_heading = "走査/入力")]
    pub text_only: bool,

    /// 高速テキスト判定（先頭 1024B, NUL 検出）。false なら厳密。
    #[arg(long, default_value_t = true, help_heading = "走査/入力")]
    pub fast_text_detect: bool,

    /// ファイル一覧を読み込む (改行区切り)
    #[arg(long, value_hint = ValueHint::FilePath, help_heading = "走査/入力")]
    pub files_from: Option<PathBuf>,

    /// ファイル一覧を読み込む (NUL 区切り)
    #[arg(long, value_hint = ValueHint::FilePath, help_heading = "走査/入力")]
    pub files_from0: Option<PathBuf>,

    /// 隠しファイルも対象
    #[arg(long, help_heading = "走査/入力")]
    pub hidden: bool,

    /// シンボリックリンクを辿る
    #[arg(long, help_heading = "走査/入力")]
    pub follow: bool,

    /// .gitignore を尊重 (git ls-files ベース)
    #[arg(long, help_heading = "走査/入力")]
    pub git: bool,

    /// 並列数（1..=512）
    #[arg(long, help_heading = "走査/入力")]
    pub jobs: Option<usize>,

    /// 既定の剪定を無効化
    #[arg(long, help_heading = "走査/入力")]
    pub no_default_prune: bool,

    /// 絶対パス出力（論理的：シンボリック解決なし）
    #[arg(long, help_heading = "パス出力")]
    pub abs_path: bool,

    /// 絶対パスを実体解決（canonicalize）で出力
    #[arg(long, help_heading = "パス出力")]
    pub abs_canonical: bool,

    /// パス先頭を削除
    #[arg(long, value_hint = ValueHint::DirPath, help_heading = "パス出力")]
    pub trim_root: Option<PathBuf>,

    /// CSV/TSV 末尾に TOTAL 行を出力
    #[arg(long, help_heading = "出力")]
    pub total_row: bool,

    /// 指定日時以降 (RFC3339 / %Y-%m-%d %H:%M:%S / %Y-%m-%d)
    #[arg(long, help_heading = "フィルタ")]
    pub mtime_since: Option<DateTimeArg>,

    /// 指定日時以前
    #[arg(long, help_heading = "フィルタ")]
    pub mtime_until: Option<DateTimeArg>,

    /// 改行も文字数に含める（直感的カウント）
    #[arg(long, help_heading = "出力")]
    pub count_newlines_in_chars: bool,

    /// 進捗表示（非TTY/非table出力では内部で無効化推奨）
    #[arg(long, help_heading = "出力")]
    pub progress: bool,

    /// フィルタ式（例: "lines > 100 && ext == 'rs'")
    #[arg(long, help_heading = "フィルタ")]
    pub filter: Option<String>,

    /// 比率列を追加（一覧/集計で%)
    #[arg(long, help_heading = "出力")]
    pub ratio: bool,

    /// 出力先ファイル（未指定は標準出力）
    #[arg(long, value_hint = ValueHint::FilePath, help_heading = "出力")]
    pub output: Option<PathBuf>,

    /// 厳格モード（1件失敗で終了）。既定は警告して続行
    #[arg(long, help_heading = "動作")]
    pub strict: bool,

    /// 比較: 2つの JSON を比較表示
    #[arg(long, num_args = 2, value_names = ["OLD", "NEW"], value_hint = ValueHint::FilePath, help_heading = "比較")]
    pub compare: Option<Vec<PathBuf>>,

    /// 対象パス
    #[arg(value_hint = ValueHint::AnyPath, help_heading = "走査/入力")]
    pub paths: Vec<PathBuf>,
}
