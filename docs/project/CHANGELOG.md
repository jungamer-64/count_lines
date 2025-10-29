# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `--sort size` を追加し、ファイルサイズでのソートをサポートしました。行数/文字数/単語数と同様に `:desc` 修飾子で降順指定が可能です。
- `--incremental` / `--cache-dir` オプションを追加し、キャッシュを利用した差分計測をサポートしました。小規模変更時の再実行が高速化されます。
- `--watch` / `--watch-interval` / `--watch-output` を追加し、ファイル変更を監視しながらインクリメンタル再計測できるウォッチモードを導入しました。`--watch-output jsonl` では各リフレッシュ結果をサマリに加えて `changed_files` / `removed_files` を含む JSON Lines で受け取れます。
- キャッシュ検証/管理のための `--cache-verify` / `--clear-cache` を追加し、mtime 精度やキャッシュリセットに対応しました。

## [0.7.0] - 2025-10-29

### Added
- Domain value objects (`LineCount`, `WordCount`, `FilePath` など) と `FileStatsBuilder` を導入し、統計値の構築を型安全にできるようにしました
- `ARCHITECTURE.md` を追加し、プロジェクトの設計とレイヤ構造をドキュメント化
- 開発支援スクリプト (`test.sh`, `benchmark.sh`, `release.sh`) や共通テストフィクスチャ群を追加
- `.gitignore` を拡充し、ログやビルド成果物を標準で除外

### Changed
- プロジェクト構成を再編し、ドキュメントを `docs/` 配下へ移動、テストを `cli/`・`integration/`・`fixtures/` へ整理
- 計測パイプラインを再実装し、小規模入力時の逐次処理と並列処理の切り替え、進捗通知、厳格モードでの失敗伝播を改善
- ソート処理を新しい `SortStrategy` に刷新し、複数キーやワード数ソートを値オブジェクトベースで安定処理
- CLI 構成ローダーを更新し、`--min-words` / `--max-words` / `--sort words` / `--filter` 内の `words` 参照時に単語数計測を自動有効化
- `--abs-canonical` を単独指定しても絶対パスを出力するように挙動を調整
- `.gitignore` や構成ファイルの整理に合わせてテストユーティリティ (`TempDir`, `TempWorkspace`, アサーションマッチャー) を共通化
- CLI で `--top` や `--by-limit` に 0 を指定した場合、また `--jobs` の上限外指定時に即座にエラーを返すようバリデーションを強化

### Fixed
- 厳格モード (`--strict`) で読み取り不能ファイルが存在する場合に確実にエラーとするよう修正し、非厳格モードでは警告を出して継続
- フィルタ式のパースエラーをユーザーに明示的に報告し、無音でレコードが欠落しないように修正
- 行数・文字数・単語数レンジフィルタが値オブジェクト経由でも正しく適用されるよう整備

## [0.5.0] - 2024-10-27

### Added
- Layered architecture (shared & domain core ← application ← presentation/bootstrap, with infrastructure adapters)
- Workspace configuration with separate core library (`count_lines_core`)
- JSON/YAML/JSONL output with version field for snapshot comparison
- Snapshot comparison feature (`--compare old.json new.json`)
- Grouping by extension, directory depth, and mtime (`--by`)
- Expression-based filtering (`--filter "lines > 100 && ext == 'rs'"`)
- Progress indicator support (`--progress`)
- Ratio columns for percentage display (`--ratio`)
- Git mode with `.gitignore` support (`--git`)
- Multiple output formats: Table, CSV, TSV, JSON, YAML, Markdown, JSONL
- Comprehensive filtering options (size, lines, chars, words, mtime, glob)
- Multi-field sorting with ascending/descending order
- Word count support (`--words`)
- Hidden files option (`--hidden`)
- File list input support (`--files_from`)

### Changed
- Refactored to clean architecture with clear layer separation
- Improved parallel processing with Rayon
- Enhanced error handling with anyhow
- Better CLI argument parsing with clap v4.5

### Performance
- Optimized release builds with LTO and single codegen unit
- Parallel file processing for large repositories
- Efficient text file detection

## [0.4.0] - (Historical)

### Added
- Basic file counting functionality
- Multiple output format support
- Sorting capabilities

## [0.3.0] - (Historical)

### Added
- Recursive directory scanning
- Extension-based filtering
- CSV output support

## [0.2.0] - (Historical)

### Added
- Character counting
- Basic filtering options

## [0.1.0] - (Historical)

### Added
- Initial release
- Basic line counting for files
- Simple CLI interface

---

## Categories

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security improvements
- **Performance**: Performance improvements

[Unreleased]: https://github.com/jungamer-64/count_lines/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/jungamer-64/count_lines/compare/v0.5.0...v0.7.0
[0.5.0]: https://github.com/jungamer-64/count_lines/releases/tag/v0.5.0
