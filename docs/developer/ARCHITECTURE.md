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
├── crates/                         # ワークスペースメンバー（レイヤー別クレート）
│   ├── shared-kernel/              # 共有カーネル（エラー型・値オブジェクト）
│   ├── domain/                     # ドメインモデル・ビジネスルール
│   ├── ports/                      # ポート定義（インターフェース trait）
│   ├── usecase/                    # ユースケース・オーケストレーション
│   ├── infra/                      # インフラ基盤実装
│   └── core/                       # 統合レイヤー + アプリケーション固有実装
│       ├── src/
│       │   ├── application/        # CQRS (commands/queries) - CLI 固有
│       │   ├── bootstrap.rs        # DI 構築・アプリケーション起動
│       │   ├── infrastructure/     # CLI 固有アダプター (出力フォーマッタ等)
│       │   ├── shared/             # 共通ユーティリティ
│       │   ├── lib.rs              # 各クレートの re-export
│       │   └── version.rs          # バージョン情報
│       └── Cargo.toml
├── src/                            # メインクレート（CLI バイナリ）
│   ├── lib.rs                      # core の再エクスポート + CLI モジュール公開
│   ├── main.rs                     # CLI エントリポイント
│   └── cli/                        # clap 引数定義・パーサー
├── tests/
│   ├── common/                     # テストユーティリティ
│   ├── unit/                       # ユニットテスト（レイヤー別）
│   ├── integration/                # 統合テスト
│   └── e2e/                        # エンドツーエンドテスト
├── benches/                        # ベンチマーク
├── scripts/                        # 開発・CI スクリプト
├── docs/                           # ドキュメント (user/developer/project)
├── .config/                        # ツール構成 (rustfmt/clippy/CI/codacy)
└── Cargo.toml                      # ワークスペース設定
```

### ワークスペース構成

プロジェクトは**マルチクレート構成**を採用し、クリーンアーキテクチャのレイヤーを物理的に分離しています。

| クレート | 役割 | 依存先 |
|----------|------|--------|
| `count_lines_shared_kernel` | 共有エラー型・値オブジェクト | なし（最内層）|
| `count_lines_domain` | ドメインモデル・ビジネスルール | shared-kernel |
| `count_lines_ports` | インターフェース定義（trait） | shared-kernel |
| `count_lines_usecase` | 汎用ユースケース・オーケストレーション | domain, ports |
| `count_lines_infra` | インフラ基盤実装（ファイルシステム等）| domain, ports |
| `count_lines_core` | **統合レイヤー** + CLI 固有実装 | 全クレート |
| `count_lines` | CLI バイナリ | core |

#### `core` クレートの役割

`core` は単なる再エクスポートではなく、**CLI アプリケーション固有の実装**も含むハイブリッドクレートです：

- **bootstrap**: DI コンテナの構築とアプリケーション起動
- **application/commands**: `RunAnalysisCommand`, `RunAnalysisHandler`（CQRS コマンド）
- **application/queries**: `ConfigQueryService`（設定構築クエリ）
- **infrastructure/adapters**: CLI 固有のアダプター（出力フォーマッタ、通知等）

#### `infra` vs `core/infrastructure` の使い分け

| 場所 | 内容 | 目的 |
|------|------|------|
| `crates/infra/` | ファイルシステム、キャッシュ、計測など | **再利用可能な汎用基盤** |
| `crates/core/infrastructure/` | 出力フォーマッタ、比較、シリアライゼーション | **CLI アプリケーション固有** |

## レイヤー構成

### Shared Kernel (`crates/shared-kernel/`)

- 全レイヤーで共有されるエラー型階層 (`CountLinesError`, `DomainError`, `InfrastructureError` 等)
- 基本的な値オブジェクト (`LineCount`, `CharCount`, `FileSize`, `FilePath` 等)
- パス操作ユーティリティ

### Domain Layer (`crates/domain/`)

- DDD のエンティティ、値オブジェクト、ドメインサービス
- `model/`: `FileEntry`, `FileStats` などのドメインモデル
- `analytics/`: 集計・ソートといった純粋なドメインロジック
- `config/`: 設定関連のドメインモデル
- `options/`: 出力形式、ソートキーなどのオプション型

### Ports Layer (`crates/ports/`)

- インターフェース定義（trait）のみを配置
- `filesystem`: ファイルシステムアクセスの抽象化
- `hashing`: ファイルハッシュ計算の抽象化
- `progress`: 進捗通知の抽象化

### Use Case Layer (`crates/usecase/`)

- 汎用的なユースケース・オーケストレーションロジック
- `orchestrator`: ファイルカウントの主要オーケストレーション
- `dto`: ユースケース境界のデータ転送オブジェクト

### Infrastructure Layer (`crates/infra/`)

- 汎用的なインフラ実装（再利用可能）
- `filesystem/`: ファイル列挙、Git 連携
- `measurement/`: ファイル計測（行数・文字数・単語数）
- `cache/`: インクリメンタルキャッシュ
- `persistence/`: ファイル読み書きヘルパー
- `watch/`: ファイル監視サービス

### Core Layer (`crates/core/`)

統合レイヤー + CLI 固有実装を担当します。

- `bootstrap.rs`: DI 構築・アプリケーション起動
- `application/commands/`: CLI 固有の CQRS コマンド
  - `RunAnalysisCommand`, `RunAnalysisHandler`
- `application/queries/`: 設定構築クエリ
  - `ConfigQueryService`
- `infrastructure/adapters/`: CLI 固有アダプター
  - `OutputEmitter`, `ConsoleNotifier`, `JsonlWatchEmitter` 等
- `infrastructure/comparison/`: スナップショット比較
- `infrastructure/io/`: 出力フォーマット処理

### Presentation Layer (`src/cli/`)

- CLI 引数定義（`clap` ベース）
- `Args`: 引数構造体
- `build_config`: 引数から `Config` への変換

## データフロー

### 典型的な実行フロー

```text
1. CLI 入力 (src/cli/)
   ↓
2. Config クエリサービスでドメイン設定構築 (core/application/queries)
   ↓
3. ブートストラップでユースケース実行 (core/bootstrap)
   ↓
4. ファイル列挙 (infra/filesystem via ports/filesystem trait)
   ↓
5. 計測・統計値算出 (infra/measurement)
   ↓
6. ドメインサービスでソート・集計 (domain/analytics)
   ↓
7. 出力アダプターでフォーマット & 出力 (core/infrastructure/io)
```

スナップショット比較などのクエリ系ユースケースは、CQRS の考え方に基づき `SnapshotComparator` ポートを経由して処理されます。

## 主要コンポーネント

### ユースケース (`core/application/commands`)

- `RunAnalysisCommand`: 実行要求を表すシンプルなコマンド DTO（ドメイン `Config` への参照のみ保持）
- `RunAnalysisHandler`: 上記コマンドを処理し、ファイル収集→統計計算→出力までをオーケストレーション
- ポート経由で副作用を注入することで、テストではモックを差し替え可能

### クエリ (`core/application/queries`)

- `ConfigQueryService`: CLI DTO (`ConfigOptions`) からドメイン `Config` を構築
- フィルタ／ソート指定の検証と正規化を担当

### ドメインサービス (`domain/analytics` 等)

- `Aggregator`: 拡張子やディレクトリごとの集計
- `apply_sort`: 複数ソートキーに基づいた安定ソート

### インフラ基盤 (`infra/`)

- `filesystem::collect_entries`: Git 連携を含むファイル列挙
- `measurement::measure_entries`: Rayon を用いた並列計測
- `cache::CacheStore`: インクリメンタルキャッシュ管理
- `watch::WatchService`: ファイル変更監視

### CLI 固有アダプター (`core/infrastructure/`)

- `adapters::OutputEmitter`: 表形式、CSV/TSV、JSON/JSONL、Markdown など多様な出力
- `adapters::ConsoleNotifier`: 進捗通知
- `comparison::SnapshotDiffAdapter`: JSON スナップショットの差分計算

### プレゼンテーション (`src/cli/`)

- `Args`: `clap` による CLI 引数定義
- `build_config`: DTO からユースケース入力を生成

### ブートストラップ (`core/bootstrap`)

- CLI ユースケースの起動 (`run_with_config`)
- ポート実装を束ねて `RunAnalysisHandler` を生成し、`RunAnalysisCommand` を発行
- TTY 判定、バナー表示、ウォッチモード制御

## 設計原則

- **SOLID 原則** を遵守: 特に単一責務・依存逆転を強調
- **DDD**: ドメインモデルとユースケースの明確な境界を定義
- **CQRS**: コマンドとクエリを別モジュールに分離し、新しいユースケースの追加を容易に
- **Clean Architecture**: 内向き依存を徹底し、テスト容易性と拡張性を担保

この構造により、CLI 以外のフロントエンド追加や新しい出力形式の導入、あるいは別ストレージへの対応なども最小限の変更で実現可能です。
