# count_lines

高速かつ柔軟にファイル群の行数・文字数・単語数を集計する CLI ツールです。  
ディレクトリ配下を再帰的に走査して統計情報を取得し、表・CSV/TSV・Markdown・JSON/JSONL・YAML といった多彩なフォーマットで出力できます。

## 主な機能

- Rayon による並列処理で大規模リポジトリでもスピーディーに集計
- `.gitignore` を尊重する `--git` モードや豊富なフィルタ（glob / サイズ / 行数 / 文字数 / 単語数 / mtime / 式）
- 拡張子・ディレクトリ深度・更新時刻などでの集計 (`--by ext`, `--by dir=2`, `--by mtime:month`)
- JSON/YAML/JSONL 出力には `version` フィールドを含め、スナップショットの比較 (`--compare old.json new.json`) にも対応
- テキストファイル判定、進捗表示、比率列など、日々のコードメトリクス収集を支援

## インストール

### Cargo からインストール

```bash
cargo install --git https://github.com/jungamer-64/count_lines
```

### ソースからビルド

```bash
git clone https://github.com/jungamer-64/count_lines.git
cd count_lines
cargo build --release
./target/release/count_lines --help
```

## 使い方

```
count_lines [OPTIONS] [PATHS]...
```

- `PATHS` を省略するとカレントディレクトリ (`.`) が対象になります。
- すべてのオプションは `count_lines --help` または `count_lines --help --verbose` で確認できます。
- `usage.txt` には CLI の詳細なリファレンスがまとまっています。

### 代表的な例

```bash
# プロジェクト全体を走査して上位20件を表示
count_lines --top 20

# TypeScript と Rust のみ対象にし、結果を Markdown で保存
count_lines --ext ts,tsx,rs --format md > stats.md

# JSON スナップショットを生成して後から比較
count_lines --format json --output snapshot-$(date +%Y%m%d).json
count_lines --compare snapshot-20240101.json snapshot-20240401.json
```

## 主なオプション

| 分類 | 代表的なオプション | 説明 |
| ---- | ------------------ | ---- |
| 出力形式 | `--format table|csv|tsv|json|yaml|md|jsonl` | 既定は `table`。`--ratio` で比率列を追加可能 |
| ソート | `--sort lines:desc,name` | カンマ区切りで複合ソート。`desc` 指定可 |
| 集計 | `--by ext` / `--by dir=2` / `--by mtime:month` | 拡張子・ディレクトリ・更新時刻バケットでのサマリ |
| フィルタ | `--include '*.rs'` / `--min_lines 50` / `--filter "lines > 100 && ext == 'rs'"` | glob / サイズ / 行数 / 文字数 / 単語数 / mtime / 式による絞り込み |
| I/O | `--git` / `--hidden` / `--files_from list.txt` | `.gitignore` 尊重、隠しファイルを含める、ファイルリスト入力など |
| 進捗・整形 | `--progress` / `--trim_root /path/to/repo` / `--total_row` | 進捗表示やパス整形、CSV/TSV の合計行追加 |
| 比較 | `--compare old.json new.json` | 2つの JSON スナップショットの差分を表示 |

詳細は [`usage.txt`](usage.txt) を参照してください。

## JSON / YAML 出力について

JSON/YAML は以下のスキーマを持ちます。`words` や `mtime` は指定オプションに応じて省略され、`by` は集計を指定しない場合 `null` になります。

```json
{
  "version": "x.y.z",
  "files": [
    {
      "file": "src/main.rs",
      "lines": 120,
      "chars": 4560,
      "words": 890,
      "size": 7890,
      "mtime": "2024-02-02T12:34:56+09:00",
      "ext": "rs"
    }
  ],
  "summary": { "lines": 120, "chars": 4560, "words": 890, "files": 1 },
  "by": [
    {
      "label": "By Extension",
      "rows": [{ "key": "rs", "lines": 120, "chars": 4560, "count": 1 }]
    }
  ]
}
```

JSONL 出力ではファイルごとに `type = "file"` 行が並び、最後に `type = "total"` の行が追加され、ここにも `version` が含まれます。

## プロジェクト構成

- `crates/core/` ライブラリ crate。ユースケース、ドメイン、アダプターなどをレイヤー別に保持します。`src/` 直下は `bootstrap/`・`presentation/`・`application/`・`domain/`・`infrastructure/`・`shared/` に整理され、`version.rs` がトップレベルに残ります。
- `src/lib.rs` `count_lines_core` を再エクスポートし、従来通り `count_lines::` から利用できるようにします。
- `src/main.rs` CLI 用の薄いバイナリエントリで、`count_lines::run_from_cli()` を呼び出します。
- `scripts/install_count_lines.sh` バイナリをローカルに配置するための補助スクリプトです。
- `tests/` `assert_cmd` を使った CLI スモークテストを配置します。

## ライブラリとしての利用

CLI だけでなく、ライブラリとしてコアロジックを呼び出すこともできます。`count_lines::Args` を利用して設定を構築し、`count_lines::run_from_args` または `count_lines::run_with_config` を呼び出してください。

```rust
use clap::Parser;
use count_lines::{run_from_args, Args};

fn main() -> anyhow::Result<()> {
    let args = Args::parse_from(["count_lines", "--format", "json", "Cargo.toml"]);
    run_from_args(args)
}
```

## 開発

```bash
cargo fmt
cargo check
cargo test
# コアライブラリのみ検証したい場合
cargo test -p count_lines_core
```

PR や Issue はお気軽にどうぞ。実行時のシナリオや出力例を添えていただけると助かります。

## License

This project is licensed under either of

- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.
