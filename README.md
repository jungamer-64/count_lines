# count_lines

高速かつ柔軟にファイル群の行数・文字数・単語数を集計する CLI ツール

[![CI](https://github.com/jungamer-64/count_lines/workflows/CI/badge.svg)](https://github.com/jungamer-64/count_lines/actions/workflows/ci.yml)
[![Release](https://github.com/jungamer-64/count_lines/workflows/Release/badge.svg)](https://github.com/jungamer-64/count_lines/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

Rayon による並列処理で大規模リポジトリでもスピーディーに集計。  
多彩な出力フォーマット（Table / CSV / JSON / YAML / Markdown）に対応し、  
`.gitignore` を尊重する Git モードや豊富なフィルタリングオプションを搭載しています。

## 📚 ドキュメント

- **[📖 詳細な README](docs/user/README.md)** - プロジェクトの詳細情報・機能一覧
- **[🚀 使用方法](docs/user/USAGE.md)** - CLI オプションの完全リファレンス
- **[🤝 コントリビューション](docs/developer/CONTRIBUTING.md)** - 開発に参加する方法
- **[🏗️ アーキテクチャ](docs/developer/ARCHITECTURE.md)** - プロジェクト構造とデザイン
- **[📝 CHANGELOG](docs/project/CHANGELOG.md)** - 変更履歴

## ⚡ クイックスタート

### インストール

```bash
# Cargo からインストール
cargo install --git https://github.com/jungamer-64/count_lines

# または、ソースからビルド
git clone https://github.com/jungamer-64/count_lines.git
cd count_lines
cargo install --path crates/cli
```

### 基本的な使い方

```bash
# カレントディレクトリを集計
count_lines

# 上位20件を表示
count_lines --top 20

# Rust ファイルのみを対象に JSON 出力
count_lines --ext rs --format json

# Git リポジトリモード（.gitignore を尊重）
count_lines --git --top 30
```

詳細は [使用方法ドキュメント](docs/user/USAGE.md) を参照してください。

## 🌟 主な機能

- ⚡ **高速並列処理** - Rayon による並列化で大規模プロジェクトも高速集計
- 🎯 **柔軟なフィルタリング** - glob / サイズ / 行数 / 更新日時など多彩な条件
- 📊 **多様な出力形式** - Table, CSV, TSV, JSON, YAML, Markdown, JSONL
- 🔍 **Git 統合** - `.gitignore` を尊重した集計
- 📈 **集計機能** - 拡張子別・ディレクトリ別・更新時刻別のグルーピング
- 🔄 **スナップショット比較** - JSON 出力を使った履歴比較

## 📦 ライブラリとしての利用

```rust
use clap::Parser;
use count_lines_cli::args::Args;
use count_lines_cli::config::Config;
use count_lines_cli::engine;

fn main() -> anyhow::Result<()> {
    let args = Args::parse_from(["count_lines", "--format", "json", "."]);
    let config = Config::from(args);
    engine::run(&config).map(|_| ())
}
```

詳細は [ユーザードキュメント](docs/user/README.md) を参照してください。

## 📄 License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

## 🙏 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](docs/developer/CONTRIBUTING.md) for details.
