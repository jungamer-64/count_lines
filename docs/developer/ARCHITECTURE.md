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
├── crates/
│   ├── core/                       # 純粋な計算ロジック (no_std)
│   │   ├── src/lib.rs              # コアライブラリ
│   │   └── ...
│   ├── engine/                     # ファイル処理エンジン (I/O, Rayon)
│   │   ├── src/lib.rs              # エンジンエントリポイント
│   │   ├── filesystem.rs           # ファイル探索
│   │   ├── config.rs               # 設定定義
│   │   ├── stats.rs                # 統計データ構造
│   │   └── ...
│   └── cli/                        # コマンドラインインターフェース
│       ├── src/main.rs             # CLIエントリポイント
│       ├── args.rs                 # Clap定義
│       ├── presentation.rs         # 出力整形
│       └── ...
├── docs/                           # ドキュメント
├── scripts/                        # 開発スクリプト
└── Cargo.toml                      # ワークスペース設定
```

## モジュール構成

### クレート構成

| クレート | 役割 | 依存関係 |
|---------|------|----------|
| `count_lines_core` | `no_std` 環境でも動作する純粋な計算処理（行数、文字数、SLOC判定など） | なし (allocのみ) |
| `count_lines_engine` | ファイルシステム操作、並列処理、設定管理を行うライブラリ | `core`, `rayon`, `ignore` |
| `count_lines_cli` | ユーザー入出力、引数解析、結果の表示 | `engine` |

### Engine (`crates/engine`)

アプリケーションの中核ロジックを担当します。

| モジュール | 責務 |
|-----------|------|
| `config.rs` | アプリケーション全体の `Config` 構造体定義 |
| `filesystem.rs` | `ignore` クレートを使用したファイル探索 |
| `stats.rs` | `FileStats` 構造体（`PathBuf` や `SystemTime` を含む） |
| `watch.rs` | ファイルシステムの変更監視 (`notify`) |

### CLI (`crates/cli`)

ユーザーとのインターフェースを担当します。

| モジュール | 責務 |
|-----------|------|
| `args.rs` | `clap` によるコマンドライン引数定義 |
| `presentation.rs` | エンジンから受け取った結果の整形・表示 |
| `config_adapter.rs` | `clap` の引数から `engine::Config` への変換 |


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
