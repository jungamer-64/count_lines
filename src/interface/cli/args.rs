use crate::domain::grouping::ByMode;
use crate::domain::options::{OutputFormat, SortSpec};
use crate::foundation::util::{DateTimeArg, SizeArg};
use clap::Parser;
use std::path::PathBuf;

/// Top-level CLI arguments parsed via clap.
#[derive(Parser, Debug)]
#[command(
    name = "count_lines",
    version = crate::VERSION,
    about = "ファイル行数/文字数/単語数の集計ツール",
    long_about = Some(include_str!("../../../usage.txt"))
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// 出力フォーマット
    #[arg(long, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// ソートキー（複数可, 例: lines:desc,chars:desc,name）
    #[arg(long, default_value = "lines:desc")]
    pub sort: SortSpec,

    /// 上位N件のみ表示
    #[arg(long)]
    pub top: Option<usize>,

    /// サマリ軸 (ext, dir, dir=N, mtime[:day|week|month]) — 複数可
    #[arg(long)]
    pub by: Vec<ByMode>,

    /// サマリのみ表示（一覧は出力しないが By 集計は出す）
    #[arg(long)]
    pub summary_only: bool,

    /// 合計のみ表示（一覧と By 集計は出さない）
    #[arg(long)]
    pub total_only: bool,

    /// 集計テーブルの上位N件のみ表示
    #[arg(long)]
    pub by_limit: Option<usize>,

    /// 含めるファイル名パターン
    #[arg(long)]
    pub include: Vec<String>,

    /// 除外するファイル名パターン
    #[arg(long)]
    pub exclude: Vec<String>,

    /// 含めるパスパターン
    #[arg(long)]
    pub include_path: Vec<String>,

    /// 除外するパスパターン
    #[arg(long)]
    pub exclude_path: Vec<String>,

    /// 除外ディレクトリパターン
    #[arg(long)]
    pub exclude_dir: Vec<String>,

    /// 拡張子フィルタ (カンマ区切り)
    #[arg(long)]
    pub ext: Option<String>,

    /// 最大ファイルサイズ (例: 10K, 5MiB)
    #[arg(long)]
    pub max_size: Option<SizeArg>,

    /// 最小ファイルサイズ (例: 10K, 5MiB)
    #[arg(long)]
    pub min_size: Option<SizeArg>,

    /// 最小行数
    #[arg(long)]
    pub min_lines: Option<usize>,

    /// 最大行数
    #[arg(long)]
    pub max_lines: Option<usize>,

    /// 最小文字数
    #[arg(long)]
    pub min_chars: Option<usize>,

    /// 最大文字数
    #[arg(long)]
    pub max_chars: Option<usize>,

    /// 単語数も計測
    #[arg(long)]
    pub words: bool,

    /// 最小単語数
    #[arg(long)]
    pub min_words: Option<usize>,

    /// 最大単語数
    #[arg(long)]
    pub max_words: Option<usize>,

    /// テキストファイルのみ
    #[arg(long)]
    pub text_only: bool,

    /// 高速テキスト判定（先頭 1024B, NUL 検出）。false なら厳密。
    #[arg(long, default_value_t = true)]
    pub fast_text_detect: bool,

    /// ファイル一覧を読み込む (改行区切り)
    #[arg(long)]
    pub files_from: Option<PathBuf>,

    /// ファイル一覧を読み込む (NUL 区切り)
    #[arg(long)]
    pub files_from0: Option<PathBuf>,

    /// 隠しファイルも対象
    #[arg(long)]
    pub hidden: bool,

    /// シンボリックリンクを辿る
    #[arg(long)]
    pub follow: bool,

    /// .gitignore を尊重 (git ls-files ベース)
    #[arg(long)]
    pub git: bool,

    /// 並列数
    #[arg(long)]
    pub jobs: Option<usize>,

    /// 既定の剪定を無効化
    #[arg(long)]
    pub no_default_prune: bool,

    /// 絶対パス出力（論理的：シンボリック解決なし）
    #[arg(long)]
    pub abs_path: bool,

    /// 絶対パスを実体解決（canonicalize）で出力
    #[arg(long)]
    pub abs_canonical: bool,

    /// パス先頭を削除
    #[arg(long)]
    pub trim_root: Option<PathBuf>,

    /// CSV/TSV 末尾に TOTAL 行を出力
    #[arg(long)]
    pub total_row: bool,

    /// 指定日時以降 (RFC3339 / %Y-%m-%d %H:%M:%S / %Y-%m-%d)
    #[arg(long)]
    pub mtime_since: Option<DateTimeArg>,

    /// 指定日時以前
    #[arg(long)]
    pub mtime_until: Option<DateTimeArg>,

    /// 改行も文字数に含める（直感的カウント）
    #[arg(long)]
    pub count_newlines_in_chars: bool,

    /// 進捗表示
    #[arg(long)]
    pub progress: bool,

    /// フィルタ式（例: "lines > 100 && ext == 'rs'")
    #[arg(long)]
    pub filter: Option<String>,

    /// 比率列を追加（一覧/集計で%)
    #[arg(long)]
    pub ratio: bool,

    /// 出力先ファイル（未指定は標準出力）
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// 厳格モード（1件失敗で終了）。既定は警告して続行
    #[arg(long)]
    pub strict: bool,

    /// 比較: 2つの JSON を比較表示
    #[arg(long, num_args = 2)]
    pub compare: Option<Vec<PathBuf>>,

    /// 対象パス
    pub paths: Vec<PathBuf>,
}
