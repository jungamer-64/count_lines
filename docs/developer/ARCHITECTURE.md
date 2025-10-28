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
- **クリーンアーキテクチャ + DDD + CQRS** による疎結合な設計
- **ワークスペース構成** によるライブラリとバイナリの分離

## アーキテクチャパターン

### クリーンアーキテクチャ / DDD / CQRS

`count_lines` はクリーンアーキテクチャをベースに、ドメイン駆動設計 (DDD) とコマンド／クエリ責務分離 (CQRS) を組み合わせています。これにより以下のメリットが得られます。

1. **関心の分離**: コアドメインと I/O／アダプターの明確な分離
2. **テスタビリティ**: 各レイヤーを独立してテスト可能
3. **保守性**: 変更の影響範囲を最小化し、SOLID 原則を遵守
4. **拡張性**: 新しい入出力チャネルやユースケースの追加が容易

### レイヤー間の依存関係

```
┌──────────────────────────────────────────┐
│      Bootstrap Layer (bootstrap/)          │ ← 構成ルート / DI
├──────────────────────────────────────────┤
│      Presentation Layer (presentation/)    │ ← CLI などの境界
├──────────────────────────────────────────┤
│      Application Layer (application/)      │ ← ユースケース / CQRS
├──────────────────────────────────────────┤
│      Domain Layer (domain/)                │ ← コアドメイン
├──────────────────────────────────────────┤
│      Shared Kernel (shared/)               │ ← 値オブジェクト/共通ロジック
└──────────────────────────────────────────┘
           ↑
           │  Infrastructure Layer (infrastructure/) が外側から
           │  ポートを実装し依存を内側へ向ける
```

依存は内向きのみ許容されます。外側のレイヤーは内側に依存しますが、逆方向の依存は存在しません。

## プロジェクト構造

```
count_lines/
├── crates/
│   └── core/                       # コアライブラリクレート
│       ├── src/
│       │   ├── application/        # CQRS ユースケース (commands/queries)
│       │   ├── application.rs      # アプリケーション層のルートモジュール
│       │   ├── bootstrap.rs        # 構成ルート
│       │   ├── domain/             # ドメインモデル (analytics/config/model 等)
│       │   ├── domain.rs           # ドメイン層のルートモジュール
│       │   ├── infrastructure/     # 外部アダプター (filesystem/io/measurement/persistence…)
│       │   ├── infrastructure.rs   # インフラ層のルートモジュール
│       │   ├── presentation/       # CLI などの境界 (cli/)
│       │   ├── presentation.rs     # プレゼンテーション層ルート
│       │   ├── shared/             # 共通ユーティリティ
│       │   ├── shared.rs           # 共有モジュールルート
│       │   ├── lib.rs              # ライブラリルート
│       │   └── version.rs          # バージョン情報
│       └── Cargo.toml
├── src/
│   ├── lib.rs                     # count_lines_core の再エクスポート
│   └── main.rs                    # CLI バイナリエントリポイント
├── tests/
│   ├── cli.rs                     # CLI スモークテスト
│   ├── integration/               # 統合テスト (モジュール別)
│   └── unit/                      # ユニットテスト (レイヤー別)
├── scripts/
│   ├── build/                     # ビルド系スクリプト
│   ├── ci/                        # CI 用フロー
│   ├── deployment/                # 配布・インストーラ
│   ├── development/               # 開発支援ツール
│   └── performance/               # ベンチ／プロファイル
├── docs/                          # ドキュメント (user/developer/project)
├── config/                        # ツール構成 (rustfmt/clippy/CI)
└── Cargo.toml                     # ワークスペース設定
```

### ワークスペース構成

- **メインクレート** (`count_lines`): CLI バイナリを提供
- **コアライブラリ** (`count_lines_core`): ドメイン、ユースケース、アダプターを集約

## レイヤー構成

### Shared Kernel (`shared/`)

- ドメイン全体で共有されるパス操作やパターン解析などのユーティリティ
- ドメイン値オブジェクトが依存する最小限の機能のみを配置

### Domain Layer (`domain/`)

- DDD のエンティティ、値オブジェクト、ドメインサービスを保持
- `model/` に `FileEntry` や `FileStats` などの値オブジェクトを集約
- `analytics/` で集計・ソートといった純粋なドメインロジックを提供
- `config/` で設定関連のドメインモデルを管理

### Application Layer (`application/`)

- コマンド／クエリをユースケースとして実装
- `commands/` 内でポート (インターフェース) を定義し、`RunAnalysisCommand` などを提供
- `queries/` 内で `ConfigQueryService` などの読み取りユースケースを実装
- ドメインに依存するが、インフラ詳細には依存しない

### Presentation Layer (`presentation/`)

- CLI や今後追加される可能性のある UI を担当
- `cli/` モジュールで `clap` を用いた引数パースを実装し、アプリケーション層へ DTO を渡す

### Infrastructure Layer (`infrastructure/`)

- ファイルシステム、出力、シリアライゼーションなどの外部依存を実装
- アプリケーション層のポート (`FileEntryProvider`, `FileStatisticsProcessor`, `FileStatisticsPresenter`, `SnapshotComparator` など) を実装したアダプターを `adapters/` に配置
- `filesystem/` や `measurement/`, `io/output/`, `comparison/`, `serialization/` などで詳細な I/O ロジックを提供
- `persistence/` に共通のファイル読み書きヘルパー (`FileReader`, `FileWriter`) を集約し、他モジュールの重複処理を排除

### Bootstrap Layer (`bootstrap/`)

- 依存関係の組み立てとエントリポイント
- CLI から受け取った設定でポート実装を組み合わせ、アプリケーション層のコマンドを実行
- TTY 判定など環境依存の処理もここで完結させる

## データフロー

### 典型的な実行フロー

```
1. CLI 入力 (presentation/cli)
   ↓
2. Config クエリサービスでドメイン設定構築 (application/queries)
   ↓
3. エントリポイントでユースケース実行 (bootstrap)
   ↓
4. ファイル列挙 (infrastructure/filesystem via FileEntryProvider ポート)
   ↓
5. 計測・統計値算出 (infrastructure/measurement via FileStatisticsProcessor)
   ↓
6. ドメインサービスでソート・集計 (domain/analytics)
   ↓
7. 出力アダプターでフォーマット & 出力 (infrastructure/io/output via FileStatisticsPresenter)
```

スナップショット比較などのクエリ系ユースケースは、CQRS の考え方に基づき `SnapshotComparator` ポートを経由して処理されます。

## 主要コンポーネント

### ユースケース (`application/commands`)

- `RunAnalysisCommand`: 実行要求を表すシンプルなコマンド DTO（ドメイン `Config` への参照のみ保持）
- `RunAnalysisHandler`: 上記コマンドを処理し、ファイル収集→統計計算→出力までをオーケストレーション
- ポート経由で副作用を注入することで、テストではモックを差し替え可能

### クエリ (`application/queries`)

- `ConfigQueryService`: CLI DTO (`ConfigOptions`) からドメイン `Config` を構築
- フィルタ／ソート指定の検証と正規化を担当

### ドメインサービス (`domain/analytics` など)

- `Aggregator`: 拡張子やディレクトリごとの集計
- `apply_sort`: 複数ソートキーに基づいた安定ソート

### インフラアダプター (`infrastructure/`)

- `filesystem::collect_entries`: Git 連携を含むファイル列挙
- `measurement::measure_entries`: Rayon を用いた並列計測
- `io::output::emit`: 表形式、CSV/TSV、JSON/JSONL、Markdown など多様な出力
- `comparison::run`: JSON スナップショットの差分計算
- `adapters::*`: アプリケーション層ポートの具象実装

### プレゼンテーション (`presentation/cli`)

- `Args`: `clap` による CLI 引数定義
- `build_config`: DTO からユースケース入力を生成

### ブートストラップ (`bootstrap`)

- CLI ユースケースの起動 (`run`, `run_with_config`)
- ポート実装を束ねて `RunAnalysisHandler` を生成し、`RunAnalysisCommand` を発行
- 依存関係グラフの構築とヘッダー表示／進捗通知の制御

## 設計原則

- **SOLID 原則** を遵守: 特に単一責務・依存逆転を強調
- **DDD**: ドメインモデルとユースケースの明確な境界を定義
- **CQRS**: コマンドとクエリを別モジュールに分離し、新しいユースケースの追加を容易に
- **Clean Architecture**: 内向き依存を徹底し、テスト容易性と拡張性を担保

この構造により、CLI 以外のフロントエンド追加や新しい出力形式の導入、あるいは別ストレージへの対応なども最小限の変更で実現可能です。
