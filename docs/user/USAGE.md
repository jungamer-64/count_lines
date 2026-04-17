# count_lines 使い方ガイド

このページは「よく使う実行パターン」に絞った実践ガイドです。
全オプション一覧は `docs/user/CLI_REFERENCE.md` を参照してください。

## 基本

- 実行形式: `count_lines [OPTIONS] [PATHS]...`
- `PATHS` 省略時は `.` を対象
- 出力保存はリダイレクトを使用（例: `> result.json`）

## よく使う実行パターン

### 1) 全体をざっくり確認

`count_lines .`

### 2) 拡張子を限定する

`count_lines --ext rs,toml --sort lines:desc .`

### 3) ディレクトリを除外する

`count_lines --exclude "target/**" --exclude "node_modules/**" .`

### 4) サイズ・行数で絞る

`count_lines --min-size 1KiB --max-size 1MiB --min-lines 20 --max-lines 3000 .`

### 5) 単語数を基準に確認する

`count_lines --words --sort words:desc --min-words 50 .`

### 6) SLOC を確認する

`count_lines --sloc --sort sloc:desc .`

### 7) JSON スナップショットを作る

`count_lines --format json . > snapshot.json`

### 8) スナップショットを比較する

`count_lines --compare old.json new.json`

### 9) 監視して再計測する

`count_lines --watch --watch-output full .`

## 実務のコツ

- CI 連携: `--format json` で機械処理しやすくする
- 大規模リポジトリ: `--jobs` / `--walk-threads` / `--max-depth` を調整
- `.gitignore` を無視したい時: `--no-gitignore`
- CSV/TSV で総計を残したい時: `--total-row`

## オプションカテゴリ（要約）

- 出力: `--format`, `--sort`, `--total-row`, `--count-newlines-in-chars`, `--progress`
- フィルタ: `--include`, `--exclude`, `--ext`, `--min/max-size`, `--min/max-lines`, `--min/max-chars`, `--words`, `--sloc`, `--min/max-words`, `--mtime-since/until`, `--map-ext`
- 走査: `--hidden`, `--follow`, `--no-gitignore`, `--jobs`, `--max-depth`, `--walk-threads`, `--override-include`, `--override-exclude`
- 実行モード: `--strict`, `--watch`, `--watch-interval`, `--watch-output`
- 比較: `--compare <OLD> <NEW>`
