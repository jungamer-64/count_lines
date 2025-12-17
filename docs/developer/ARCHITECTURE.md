# アーキテクチャドキュメント

このドキュメントでは、`count_lines` プロジェクトの設計思想とコード構造について説明します。

## 目次

- [概要](#概要)
- [プロジェクト構造](#プロジェクト構造)
- [モジュール構成](#モジュール構成)
- [データフロー](#データフロー)
- [主要コンポーネント](#主要コンポーネント)
- [設計原則](#設計原則)

## 概要

`count_lines` は、ファイルの行数・文字数・単語数・SLOC（ソースコード行数）を高速に集計する CLI ツールです。以下の技術的特徴を持ちます：

- **Rust 2024 Edition** による型安全性とメモリ安全性
- **Rayon** による並列処理で大規模プロジェクトにも対応
- **ignore** クレートによる `.gitignore` ルールの尊重
- **bytecount** による高速なバイト/文字カウント
- **多言語SLOC対応** - 20以上のプログラミング言語のコメント構文を認識

## プロジェクト構造

```text
count_lines/
├── src/                            # メインソースコード
│   ├── main.rs                     # CLIエントリポイント
│   ├── lib.rs                      # ライブラリルート（公開API）
│   ├── args.rs                     # CLI引数定義（clap derive）
│   ├── config.rs                   # 実行時設定構造体
│   ├── engine.rs                   # ファイル処理エンジン
│   ├── filesystem.rs               # ファイルシステム探索
│   ├── stats.rs                    # 統計データ構造
│   ├── options.rs                  # 出力オプション定義
│   ├── parsers.rs                  # CLIパーサーユーティリティ
│   ├── presentation.rs             # 出力フォーマット処理
│   ├── compare.rs                  # スナップショット比較機能
│   ├── watch.rs                    # ファイル監視機能
│   ├── error.rs                    # エラー型定義
│   └── language/                   # SLOC言語別処理
│       ├── mod.rs                  # SlocProcessor enum（ディスパッチ）
│       ├── processor_trait.rs      # LineProcessor trait
│       ├── comment_style.rs        # 言語別コメントスタイル定義
│       ├── string_utils.rs         # 文字列リテラル処理ユーティリティ
│       ├── heredoc_utils.rs        # ヒアドキュメント処理
│       └── processors/             # 各言語プロセッサ実装
│           ├── c_style.rs          # C/C++/Java等
│           ├── python_style.rs     # Python
│           ├── javascript_style.rs # JavaScript/TypeScript
│           ├── ruby_style.rs       # Ruby
│           └── ...                 # その他言語
├── tests/                          # 統合テスト
│   └── e2e/                        # エンドツーエンドテスト
├── benches/                        # ベンチマーク
│   └── end_to_end.rs               # パフォーマンス計測
├── scripts/                        # 開発・検証スクリプト
├── docs/                           # ドキュメント
│   ├── user/                       # ユーザー向けドキュメント
│   ├── developer/                  # 開発者向けドキュメント
│   └── project/                    # プロジェクト管理
└── Cargo.toml                      # プロジェクト設定
```

## モジュール構成

### コアモジュール

| モジュール | 責務 |
|-----------|------|
| `args.rs` | clap による CLI 引数の定義とパース |
| `config.rs` | 引数から実行時設定（`Config`）への変換 |
| `engine.rs` | ファイル処理のオーケストレーション |
| `filesystem.rs` | ディレクトリ探索、フィルタリング |
| `stats.rs` | `FileStats` 構造体（行数・文字数等） |

### 出力モジュール

| モジュール | 責務 |
|-----------|------|
| `options.rs` | `OutputFormat`, `SortKey`, `OutputMode` 等の定義 |
| `presentation.rs` | 表形式、CSV、JSON 等への整形・出力 |
| `compare.rs` | JSON スナップショット間の差分計算 |

### SLOC処理 (`language/`)

言語別のソースコード行数（SLOC）計算を担当します。

```text
SlocProcessor (enum)
├── CStyleProcessor         # C, C++, Java, Go, etc.
├── NestingCStyleProcessor  # Rust, Kotlin, Scala (ネストコメント対応)
├── JavaScriptProcessor     # JS/TS (テンプレートリテラル対応)
├── PythonProcessor         # Python (docstring対応)
├── RubyProcessor           # Ruby (=begin/=end対応)
├── ...                     # その他20+言語
└── NoComment               # プレーンテキスト
```

各プロセッサは `LineProcessor` trait を実装し、行ごとに「コードかコメントか」を判定します。

## データフロー

```text
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Entry (main.rs)                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Args Parsing (args.rs)                                          │
│  clap によるコマンドライン引数のパース                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Config Construction (config.rs)                                 │
│  Args → Config への変換、デフォルト値適用                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Engine Orchestration (engine.rs)                                │
│  ファイル収集・並列処理・統計集計のコーディネート                    │
└─────────────────────────────────────────────────────────────────┘
          │                              │
          ▼                              ▼
┌──────────────────────┐    ┌──────────────────────┐
│  Filesystem Walking   │    │  Content Processing  │
│  (filesystem.rs)      │    │  (engine.rs)         │
│  - ignore クレート     │    │  - バイナリ判定       │
│  - パターンフィルタ    │    │  - 行/文字/単語カウント│
│  - 並列探索           │    │  - SLOC計算          │
└──────────────────────┘    └──────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Presentation (presentation.rs)                                  │
│  - ソート・集計                                                   │
│  - フォーマット（Table/CSV/JSON/Markdown）                        │
│  - ファイル出力                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 並列処理モデル

```text
                    ┌──────────────────┐
                    │   Walker Thread   │
                    │  (filesystem.rs)  │
                    └────────┬─────────┘
                             │
            crossbeam-channel (bounded: 1024)
                             │
                             ▼
              ┌──────────────────────────┐
              │   Rayon par_bridge()     │
              │   並列ファイル処理        │
              └──────────────────────────┘
                      │  │  │  │
                      ▼  ▼  ▼  ▼
              ┌──────────────────────────┐
              │  process_file() × N       │
              │  (各スレッドで独立処理)    │
              └──────────────────────────┘
```

## 主要コンポーネント

### Config (`config.rs`)

実行時の全設定を保持する中心的な構造体です。

```rust
pub struct Config {
    pub walk: WalkOptions,       // ファイル探索設定
    pub filter: FilterConfig,    // フィルタリング条件
    pub output_mode: OutputMode, // Full/Summary/TotalOnly
    pub format: OutputFormat,    // Table/CSV/JSON/etc.
    pub sort: Vec<(SortKey, bool)>, // ソート条件
    // ...
}
```

### FileStats (`stats.rs`)

各ファイルの統計情報を保持します。

```rust
pub struct FileStats {
    pub path: PathBuf,
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
    pub sloc: Option<usize>,
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub is_binary: bool,
}
```

### SlocProcessor (`language/mod.rs`)

Enum Dispatch パターンを使用し、言語別の SLOC 処理を効率的に振り分けます。動的ディスパッチ（trait object）よりもインライン化の恩恵を受けやすい設計です。

## 設計原則

### 現在の設計方針

1. **シンプルさの優先**: 単一クレート構成で理解しやすさを重視
2. **パフォーマンス重視**: `rayon` + `crossbeam-channel` による効率的な並列処理
3. **拡張性**: 新しい言語サポートは `processors/` にファイル追加で対応
4. **テスタビリティ**: 各モジュールは独立してテスト可能

### 今後の改善検討事項

以下の改善は [ROADMAP.md](../project/ROADMAP.md) で追跡しています：

- **engine.rs の責務分離**: reader/counter/walker への分割でテスタビリティ向上
- **言語定義の外部化**: `languages.toml` 等による設定ファイルベースへの移行
- **設定のBuilder化**: より柔軟な設定構築のための Builder パターン導入

---

*このドキュメントは実際のコード構造を反映しています。構造変更時には本ドキュメントも更新してください。*
