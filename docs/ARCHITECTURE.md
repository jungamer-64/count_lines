# アーキテクチャドキュメント

このドキュメントでは、`count_lines` プロジェクトの設計思想、アーキテクチャパターン、およびコード構造について説明します。

## 目次

- [概要](#概要)
- [アーキテクチャパターン](#アーキテクチャパターン)
- [プロジェクト構造](#プロジェクト構造)
- [レイヤー構成](#レイヤー構成)
- [データフロー](#データフロー)
- [主要コンポーネント](#主要コンポーネント)
- [設計原則](#設計原則)

## 概要

`count_lines` は、ファイルの行数・文字数・単語数を高速に集計する CLI ツールです。以下の技術的特徴を持ちます：

- **Rust 2024 Edition** による型安全性とパフォーマンス
- **Rayon** による並列処理で大規模プロジェクトにも対応
- **レイヤードアーキテクチャ** によるモジュール化と保守性
- **ワークスペース構成** によるライブラリとバイナリの分離

## アーキテクチャパターン

### クリーンアーキテクチャの採用

`count_lines` は、クリーンアーキテクチャの原則に従って設計されています。これにより、以下のメリットが得られます：

1. **関心の分離**: ビジネスロジックと I/O の分離
2. **テスタビリティ**: 各レイヤーを独立してテスト可能
3. **保守性**: 変更の影響範囲を最小化
4. **拡張性**: 新機能の追加が容易

### レイヤー間の依存関係

```
┌─────────────────────────────────────┐
│     Application Layer (app/)        │  ← エントリポイント
├─────────────────────────────────────┤
│     Interface Layer (interface/)    │  ← CLI インターフェース
├─────────────────────────────────────┤
│     Domain Layer (domain/)          │  ← ビジネスロジック
├─────────────────────────────────────┤
│     Foundation Layer (foundation/)  │  ← 共通基盤
└─────────────────────────────────────┘
```

依存の方向は常に上から下へ。下位レイヤーは上位レイヤーに依存しません。

## プロジェクト構造

```
count_lines/
├── crates/
│   └── core/                    # コアライブラリクレート
│       ├── src/
│       │   ├── app/            # アプリケーション層
│       │   ├── interface/      # インターフェース層
│       │   ├── domain/         # ドメイン層
│       │   ├── foundation/     # 基盤層
│       │   ├── lib.rs          # ライブラリルート
│       │   └── version.rs      # バージョン情報
│       └── Cargo.toml
├── src/
│   ├── lib.rs                  # count_lines_core の再エクスポート
│   └── main.rs                 # CLI バイナリエントリポイント
├── tests/
│   ├── cli/                    # CLI テスト
│   ├── integration/            # 統合テスト
│   └── fixtures/               # テストデータ
├── scripts/                    # ビルド・テストスクリプト
├── docs/                       # ドキュメント
└── Cargo.toml                  # ワークスペース設定
```

### ワークスペース構成

プロジェクトは Cargo ワークスペースとして構成されています：

- **メインクレート** (`count_lines`): CLI バイナリを提供
- **コアライブラリ** (`count_lines_core`): すべてのビジネスロジックを含む

この構成により、ライブラリとして再利用可能な設計になっています。

## レイヤー構成

### 1. Foundation Layer (`foundation/`)

**責務**: プロジェクト全体で使用される基盤的な型・ユーティリティ

**モジュール構成**:
```
foundation/
├── types/              # 共通型定義
│   ├── file.rs        # FileInfo 構造体
│   ├── summary.rs     # Summary 構造体
│   └── mod.rs
├── serde/             # シリアライゼーション
│   ├── json.rs        # JSON ハンドリング
│   └── mod.rs
├── util/              # ユーティリティ関数
│   └── mod.rs
└── mod.rs
```

**主要な型**:
- `FileInfo`: 個別ファイルの統計情報
- `Summary`: 集計結果のサマリ
- `GroupedSummary`: グルーピングされた集計結果

### 2. Domain Layer (`domain/`)

**責務**: ビジネスロジック・ドメイン知識の実装

**モジュール構成**:
```
domain/
├── files/             # ファイル操作
│   ├── git.rs        # Git 統合（.gitignore サポート）
│   ├── inputs.rs     # ファイル入力
│   ├── matcher.rs    # ファイルマッチング（glob など）
│   ├── metadata.rs   # メタデータ収集
│   └── mod.rs
├── compute/           # 計算ロジック
│   ├── process.rs    # ファイル処理（並列化）
│   ├── aggregate.rs  # 集計ロジック
│   ├── sort.rs       # ソート処理
│   └── mod.rs
├── output/            # 出力フォーマット
│   ├── table.rs      # テーブル形式
│   ├── delimited.rs  # CSV/TSV
│   ├── json.rs       # JSON
│   ├── yaml.rs       # YAML
│   ├── jsonl.rs      # JSONL
│   ├── markdown.rs   # Markdown
│   ├── writer.rs     # 出力ライター
│   └── mod.rs
├── compare/           # スナップショット比較
│   ├── snapshot.rs   # 比較ロジック
│   └── mod.rs
├── config/            # 設定管理
│   ├── filters.rs    # フィルタ設定
│   └── mod.rs
├── grouping.rs        # グルーピングロジック
├── options.rs         # オプション定義
└── mod.rs
```

**主要な責務**:
- **files/**: ファイルシステムとの対話、Git 統合
- **compute/**: 並列処理、集計、ソート
- **output/**: 各種フォーマットへの出力変換
- **compare/**: スナップショット間の差分計算
- **config/**: 実行時設定の管理

### 3. Interface Layer (`interface/`)

**責務**: 外部との境界、CLI インターフェース

**モジュール構成**:
```
interface/
├── cli/
│   ├── args.rs         # CLI 引数定義（clap）
│   ├── value_enum.rs   # 列挙型の定義
│   └── mod.rs
└── mod.rs
```

**主要な責務**:
- CLI 引数のパース（`clap` を使用）
- ユーザー入力の検証
- Domain 層の `Config` への変換

### 4. Application Layer (`app/`)

**責務**: アプリケーション全体の調整・オーケストレーション

**モジュール構成**:
```
app/
└── mod.rs              # メイン実行ロジック
```

**主要な関数**:
- `run()`: メインの実行関数
- `run_with_config()`: 設定オブジェクトを受け取って実行

このレイヤーは、各レイヤーを組み合わせて一つの機能を実現します。

## データフロー

### 典型的な実行フロー

```
1. CLI 引数パース (interface/cli)
   ↓
2. Config オブジェクト構築 (domain/config)
   ↓
3. ファイル収集 (domain/files)
   ├─ Git モード判定
   ├─ glob マッチング
   └─ フィルタリング
   ↓
4. 並列処理 (domain/compute/process)
   ├─ Rayon による並列化
   ├─ ファイル読み込み
   └─ 行数・文字数・単語数カウント
   ↓
5. 集計・ソート (domain/compute)
   ├─ グルーピング（拡張子別など）
   ├─ ソート処理
   └─ サマリ計算
   ↓
6. 出力 (domain/output)
   ├─ フォーマット選択
   └─ ファイル/標準出力へ書き込み
```

### スナップショット比較モードのフロー

```
1. 2つの JSON ファイルを読み込み (domain/compare)
   ↓
2. バージョン検証
   ↓
3. ファイルごとの差分計算
   ↓
4. 変更サマリ作成
   ↓
5. 結果出力 (domain/output)
```

## 主要コンポーネント

### 並列処理エンジン

**場所**: `domain/compute/process.rs`

**技術**: Rayon による並列イテレータ

```rust
files.par_iter()  // 並列イテレータ
    .map(|path| process_file(path))
    .collect()
```

**特徴**:
- CPU コア数に応じた自動並列化
- 大規模ディレクトリでの高速処理
- エラーハンドリングと進捗表示の統合

### フィルタリングシステム

**場所**: `domain/config/filters.rs`, `domain/files/matcher.rs`

**サポートするフィルタ**:
- **Glob パターン**: `--include`, `--exclude`
- **拡張子**: `--ext rs,toml`
- **サイズ**: `--min-size`, `--max-size`
- **行数/文字数/単語数**: `--min-lines`, `--max-chars` など
- **更新時刻**: `--mtime-after`, `--mtime-before`
- **式評価**: `--filter "lines > 100 && ext == 'rs'"`

### 出力フォーマッター

**場所**: `domain/output/`

**サポートフォーマット**:
- Table: 整形されたテーブル表示
- CSV/TSV: スプレッドシート互換
- JSON/YAML: 構造化データ（バージョン情報付き）
- JSONL: ストリーミング向けライン区切り JSON
- Markdown: ドキュメント埋め込み用

各フォーマッターは統一された `Writer` trait を実装し、ポリモーフィズムを活用しています。

### Git 統合

**場所**: `domain/files/git.rs`

**機能**:
- `.gitignore` の自動検出とパース
- Git 管理下でないファイルの除外
- サブモジュール対応

## 設計原則

### 1. 型安全性

Rust の型システムを最大限活用し、コンパイル時に多くのエラーを検出します。

```rust
// 例: 列挙型で出力フォーマットを表現
pub enum OutputFormat {
    Table,
    Csv,
    Tsv,
    Json,
    Yaml,
    Markdown,
    Jsonl,
}
```

### 2. エラーハンドリング

`anyhow::Result` を使用し、エラーコンテキストを保持します。

```rust
use anyhow::{Context, Result};

fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))
}
```

### 3. テスタビリティ

各レイヤーを独立してテスト可能にするため、依存性の注入を活用します。

```
tests/
├── cli/           # CLI レベルのテスト
├── integration/   # 統合テスト
└── fixtures/      # テストデータ
```

### 4. パフォーマンス

- **並列処理**: Rayon による CPU 効率の最大化
- **ゼロコピー**: 可能な限り `&str` や `&[u8]` を使用
- **遅延評価**: イテレータの活用
- **LTO**: リリースビルドで Link Time Optimization を有効化

### 5. 拡張性

新しい出力フォーマットやフィルタの追加が容易な設計：

```rust
// 新しいフォーマッターの追加例
impl Writer for NewFormatter {
    fn write_header(&mut self) -> Result<()> { /* ... */ }
    fn write_file(&mut self, file: &FileInfo) -> Result<()> { /* ... */ }
    fn write_summary(&mut self, summary: &Summary) -> Result<()> { /* ... */ }
}
```

## バージョニング

JSON/YAML 出力には `version` フィールドを含めることで、将来的な互換性問題に対応しています。

```json
{
  "version": "0.5.0",
  "files": [ /* ... */ ],
  "summary": { /* ... */ }
}
```

スナップショット比較時にバージョンの不一致を検出し、警告を表示します。

## 今後の拡張性

現在のアーキテクチャは、以下の拡張に対応可能です：

1. **新しい言語サポート**: 言語別のコメント除去などの特殊処理
2. **プラグインシステム**: カスタムフィルタや出力フォーマット
3. **Web API**: REST API としての公開
4. **GUI フロントエンド**: コアライブラリの再利用
5. **データベース統合**: 履歴データの保存・分析

## 参考リソース

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clean Architecture (Robert C. Martin)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rayon Documentation](https://docs.rs/rayon/)
- [clap Documentation](https://docs.rs/clap/)

---

**最終更新**: 2024-10-28  
**対応バージョン**: 0.5.0