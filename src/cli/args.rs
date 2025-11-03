// src/cli/args.rs
use std::path::PathBuf;

use clap::{Parser, ValueHint};
use count_lines_core::domain::{grouping::ByMode, options::SortSpec};

use super::{
    parsers::{DateTimeArg, SizeArg},
    value_enum::{CliOutputFormat, CliWatchOutput},
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
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// 出力フォーマット
    #[arg(long, value_enum, default_value = "table", help_heading = "出力")]
    pub format: CliOutputFormat,

    /// ソートキー（複数可, 例: lines:desc,chars:desc,name）。`words` を含む場合は単語数計測が自動有効化されます。
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

    /// ディレクトリへの降下のみを抑制（ファイルが直接指定された場合は許可）
    #[arg(long = "exclude-dir-only", value_delimiter = ',', help_heading = "フィルタ")]
    pub exclude_dir_only: Vec<String>,

    /// 無視リストで再包含するパターン（gitignore 互換、カンマ区切り/複数指定可）
    #[arg(long = "override-include", value_delimiter = ',', help_heading = "走査/入力")]
    pub override_include: Vec<String>,

    /// 無視リストで追加除外するパターン（gitignore 互換、カンマ区切り/複数指定可）
    #[arg(long = "override-exclude", value_delimiter = ',', help_heading = "走査/入力")]
    pub override_exclude: Vec<String>,

    /// 拡張子フィルタ（カンマ区切り/複数指定可, 例: --ext rs,js --ext ts）
    #[arg(long, value_delimiter = ',', help_heading = "フィルタ")]
    pub ext: Vec<String>,

    /// テキスト扱いを強制する拡張子（カンマ区切り/複数指定可）
    #[arg(long = "force-text-ext", value_delimiter = ',', help_heading = "フィルタ")]
    pub force_text_ext: Vec<String>,

    /// バイナリ扱いを強制する拡張子（カンマ区切り/複数指定可）
    #[arg(long = "force-binary-ext", value_delimiter = ',', help_heading = "フィルタ")]
    pub force_binary_ext: Vec<String>,

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

    /// 最小単語数（指定すると --words が暗黙に有効化されます。CLI では --min-words）
    #[arg(long, help_heading = "フィルタ")]
    pub min_words: Option<usize>,

    /// 最大単語数（指定すると --words が暗黙に有効化されます。CLI では --max-words）
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

    /// パスの大小文字を区別せず巡回結果を重複排除
    #[arg(long, help_heading = "走査/入力")]
    pub case_insensitive_dedup: bool,

    /// シンボリックリンクを辿る
    #[arg(long, help_heading = "走査/入力")]
    pub follow: bool,

    /// .gitignore を尊重 (git ls-files ベース)
    #[arg(long, help_heading = "走査/入力", conflicts_with = "no_gitignore")]
    pub git: bool,

    /// .gitignore を無視して巡回
    #[arg(long, help_heading = "走査/入力", conflicts_with = "git")]
    pub no_gitignore: bool,

    /// 並列数（1..=512）
    #[arg(long, value_parser = clap::value_parser!(usize), help_heading = "走査/入力")]
    pub jobs: Option<usize>,

    /// ディレクトリ探索の最大深さ
    #[arg(long = "max-depth", help_heading = "走査/入力")]
    pub max_depth: Option<usize>,

    /// ファイル探索に使うスレッド数（1..=512）
    #[arg(long = "walk-threads", value_parser = clap::value_parser!(usize), help_heading = "走査/入力")]
    pub walk_threads: Option<usize>,

    /// 既定の剪定を無効化
    #[arg(long, help_heading = "走査/入力")]
    pub no_default_prune: bool,

    /// 絶対パス出力（論理的：シンボリック解決なし）
    #[arg(long, help_heading = "パス出力")]
    pub abs_path: bool,

    /// 絶対パスを実体解決（canonicalize）で出力（単独指定でも絶対化されます）
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

    /// フィルタ式（例: "lines > 100 && ext == 'rs'"）。`words` を参照すると単語数計測が自動有効化されます。
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

    /// インクリメンタルキャッシュを利用して差分のみを再計測
    #[arg(long, help_heading = "動作")]
    pub incremental: bool,

    /// キャッシュ保存先を明示的に指定
    #[arg(long, value_hint = ValueHint::DirPath, help_heading = "動作")]
    pub cache_dir: Option<PathBuf>,

    /// キャッシュを検証する際にハッシュを使用
    #[arg(long, help_heading = "動作")]
    pub cache_verify: bool,

    /// キャッシュを削除してから実行
    #[arg(long, help_heading = "動作")]
    pub clear_cache: bool,

    /// 変更を監視して継続的に再実行
    #[arg(short = 'w', long, help_heading = "動作")]
    pub watch: bool,

    /// 監視イベントのデバウンス/ポーリング間隔 (秒)（1..=86400）
    #[arg(long, help_heading = "動作")]
    pub watch_interval: Option<u64>,

    /// watch時の出力モード
    #[arg(long, value_enum, default_value = "full", help_heading = "動作")]
    pub watch_output: CliWatchOutput,

    /// 比較: 2つの JSON を比較表示
    #[arg(long, num_args = 2, value_names = ["OLD", "NEW"], value_hint = ValueHint::FilePath, help_heading = "比較")]
    pub compare: Option<Vec<PathBuf>>,

    /// 対象パス
    #[arg(value_hint = ValueHint::AnyPath, help_heading = "走査/入力")]
    pub paths: Vec<PathBuf>,
}
