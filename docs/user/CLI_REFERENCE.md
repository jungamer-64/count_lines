# count_lines CLI リファレンス

`count_lines` は、ファイルの `lines/chars` を基本に、必要に応じて `words/sloc` を計測する CLI です。

## 使い方

`count_lines [OPTIONS] [PATHS]...`

- `PATHS` を省略すると `.` が対象
- 標準出力へ出すため、ファイル保存はシェルのリダイレクト（`>`）を使用

## 出力関連

- `--format <table|csv|tsv|json|yaml|md|jsonl>`
- `--sort <SPEC>`（例: `lines:desc,chars:desc,name`）
- `--total-row`（CSV/TSV の末尾に `TOTAL` 行を追加）
- `--count-newlines-in-chars`（改行を文字数に含める）
- `--progress`

### ソートキー

`lines`, `chars`, `words`, `size`, `name`, `ext`, `sloc`

## フィルタ関連

- `--include <PATTERN>` / `--exclude <PATTERN>`（複数指定可）
- `--ext <EXTS>`（カンマ区切り。例: `rs,py,toml`）
- `--max-size <SIZE>` / `--min-size <SIZE>`
- `--min-lines <N>` / `--max-lines <N>`
- `--min-chars <N>` / `--max-chars <N>`
- `--words` / `--sloc`
- `--min-words <N>` / `--max-words <N>`
- `--mtime-since <DATETIME>` / `--mtime-until <DATETIME>`
- `--map-ext <ext=lang>`（複数指定可。例: `h=cpp`）

### 注意

- `--min-words` / `--max-words` を使うと単語数計測が有効化されます
- `--sloc` が未指定のとき SLOC 列は出力しません

## 走査関連

- `--hidden`
- `--follow`
- `--no-gitignore`
- `--jobs <N>`
- `--max-depth <N>`
- `--walk-threads <N>`
- `--override-include <PATTERN>` / `--override-exclude <PATTERN>`

## 実行モード

- `--strict`
- `-w, --watch`
- `--watch-interval <SECS>`
- `--watch-output <full|jsonl>`

## 比較

- `--compare <OLD> <NEW>`

`OLD` と `NEW` は `--format json` で出力したファイルを想定します。

## 出力フォーマット補足

- `table`: 人間向けの表
- `csv` / `tsv`: ヘッダー付き
- `json` / `yaml`: ファイル配列をそのまま出力
- `md`: Markdown テーブル
- `jsonl`: ファイル行 + 末尾に `type=total` 行

## 実用例

1. カレントディレクトリを JSON で保存

   `count_lines --format json . > snapshot.json`

2. Rust/TOML のみ、行数降順

   `count_lines --ext rs,toml --sort lines:desc .`

3. 大きいファイルを除外して CSV 出力

   `count_lines --max-size 1MiB --format csv --total-row . > stats.csv`

4. 監視モード

   `count_lines --watch --watch-output full .`

5. スナップショット比較

   `count_lines --compare old.json new.json`
