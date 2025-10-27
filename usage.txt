# count_lines — ファイル行数/文字数/単語数の集計ツール

Version: 2.4.2

## 概要

ディレクトリ以下のファイルを走査し、各ファイルの行数・文字数（必要に応じて単語数）を測定して一覧・集計します。
出力形式は表/CSV/TSV/Markdown/JSON/JSONL/YAML に対応。拡張子・ディレクトリ・更新時刻（粒度指定可）での集計も可能です。

## 主な特徴

* 高速な並列走査（Rayon）とフィルタリング
* `.gitignore` 尊重モード（`--git`）
* 複合ソート（例: `--sort lines:desc,name`）
* 多彩なフィルタ（サイズ/行数/文字数/単語数/式/mtime 範囲）
* サマリ集計（`--by ext` / `--by dir=2` / `--by mtime:month` など）
* JSON/YAML 出力に `version` フィールドを含む（機械処理向け）
* JSONL 出力の末尾 `total` 行にも `version` を付与

## 変更点（2.4.2）

* **mtime 集計に `"(no mtime)"` バケットを追加**（mtime が取得できないファイルも集計対象に）
* **比較出力（`--compare`）を拡充**：Lines/Chars/Files の差分を常に表示。さらに**両スナップショットが `words` を含む場合のみ** Words の差分も表示
* 内部リファクタで安定性/可読性向上（外部仕様は互換）

## 使い方（シノプシス）

```
count_lines [OPTIONS] [PATHS]...
```

PATHS を省略するとカレントディレクトリ（`.`）を対象にします。

## 主なオプション

### 出力形式

* `--format {table|csv|tsv|json|yaml|md|jsonl}`  … 既定は `table`

### ソート

* `--sort <SPEC>`  … 既定は `lines:desc`

  * `SPEC` はカンマ区切りの複数キー。各キーは `:desc` 指定可。
  * 例: `--sort lines:desc,chars:desc,name`
  * キー: `lines`, `chars`, `words`, `name`, `ext`
  * **安定ソート**を「**最後に書いたキーから**」適用します。

### 件数制限

* `--top <N>`  … 一覧の上位 N 件のみ表示（集計には影響なし）

### サマリ集計

* `--by <MODE>`  … 複数指定可

  * `ext`                  … 拡張子別
  * `dir` or `dir=N`       … ディレクトリ（N = 深さ、省略時1）
  * `mtime[:day|week|month]` … 変更時刻の粒度バケット集計（既定 `day`）
* `--by_limit <N>` … 各集計テーブルの上位 N 行に制限
  **注**：`mtime` 集計は **mtime の無いファイルを `"(no mtime)"`** として集計します。

### 出力抑制

* `--summary_only` … 一覧を出さず、集計（by）+ 合計のみを出力
* `--total_only`   … 一覧と集計を出さず、合計のみを出力

### フィルタ（名前/パス/拡張子）

* `--include <GLOB>` … ファイル名に対する include（複数可）
* `--exclude <GLOB>` … ファイル名に対する exclude（複数可）
* `--include_path <GLOB>` / `--exclude_path <GLOB>` … パス全体に適用
* `--exclude_dir <GLOB>` … ディレクトリ名に適用（走査除外）
* `--ext <EXTS>` … 拡張子フィルタ（**カンマ区切り・大文字小文字無視** / 例: `rs,py,ts`）
  ※ GLOB は `glob` 準拠。`*`, `?`, `[...]`, `**`（パス用）など。

### フィルタ（数値/サイズ/時刻）

* `--max_size <SIZE>` / `--min_size <SIZE>`（例: `10K`, `5MiB`）
* `--min_lines <N>` / `--max_lines <N>`
* `--min_chars <N>` / `--max_chars <N>`
* `--words` を付けた時のみ: `--min_words <N>` / `--max_words <N>`
* mtime 範囲: `--mtime_since <DATE|DATETIME>` / `--mtime_until <DATE|DATETIME>`
  *受理形式*: RFC3339 / `YYYY-MM-DD HH:MM:SS` / `YYYY-MM-DD`

### 式フィルタ

* `--filter "<expr>"`

  * 使用可能な変数: `lines`, `chars`, `words`, `size`, `ext`, `name`, `mtime`（UNIX秒）
  * 例:
    `--filter "lines > 100 && ext == 'rs'"`
    `--filter "(mtime >= 1700000000) && (chars < 2000)"`

### テキスト判定

* `--text_only` … テキストと判定されたファイルのみ対象
* `--fast_text_detect[=true|false]`（既定: true）

  * true: 先頭 1024 バイトで NUL 検出（高速・ごく稀に誤判定あり）
  * false: 全読みで NUL 検出（厳密だがメモリ/時間コスト増）

### I/O・走査

* `--files_from <PATH>`   … 改行区切りのファイルリストから読む
* `--files_from0 <PATH>`  … NUL 区切りのファイルリストから読む
* `--hidden`              … 隠しファイルも対象
* `--follow`              … シンボリックリンクを辿る
* `--git`                 … `.gitignore` を尊重（`git ls-files` ベース）
* `--no_default_prune`    … 既定の除外を無効化
  既定で除外するディレクトリ：
  `.git`, `.hg`, `.svn`, `node_modules`, `.venv`, `venv`, `build`, `dist`, `target`,
  `.cache`, `.direnv`, `.mypy_cache`, `.pytest_cache`, `coverage`, `__pycache__`,
  `.idea`, `.next`, `.nuxt`
* `--jobs <N>`            … 並列数（既定＝**論理 CPU 数**）
* `--progress`            … 簡易進捗を標準エラーに表示

### 出力オプション

* `--ratio`                 … 一覧/集計に % 列を追加（table/md）
* `--output <PATH>`         … 出力先ファイル（既定は標準出力）
* `--abs_path` / `--abs_canonical` … パスの絶対化（論理/実体解決）
  **注**：`--abs_canonical` は **`--abs_path` と併用した場合のみ有効**
* `--trim_root <PATH>`      … 表示パスの先頭から `<PATH>` を取り除く
* `--total_row`             … CSV/TSV の末尾に `TOTAL` 行を追加
* `--words`                 … 単語数を測定（一覧/集計/JSON 系に反映）
* `--count_newlines_in_chars` … 改行も文字数に含める（通常は除外）

### 厳格モード

* `--strict`  … 測定中の 1 件エラーで即終了（既定は警告して続行）

### 比較モード

* `--compare <OLD> <NEW>` … 2 つの JSON スナップショットを比較

  * 例:

    ```bash
    count_lines --format json > old.json
    # 作業…
    count_lines --format json > new.json
    count_lines --compare old.json new.json
    ```
  * 出力：**サマリ差分**（Lines/Chars/Files）と **変更ファイル一覧**
    両 JSON が `words` を含む場合は **Words の差分**も併せて表示

## 出力形式の詳細

### TABLE（既定）

* 列: `LINES [%,オプション] | CHARS [%,オプション] | WORDS(オプション) | FILE`
* `--summary_only` 時は一覧を出さず、集計（by）と合計のみ表示
* `--ratio` で % 列を追加

### CSV/TSV

* ヘッダ行あり。パス中の区切り文字は適切にクオート
* `--total_row` 指定で最終行に `TOTAL` を追加

### Markdown

* 一覧は Markdown テーブル
* `--ratio` で % 列を追加
* 集計（by）は `###` 見出しの下にテーブルで出力

### JSON

* ルートオブジェクト：

  ```json
  {
    "version": "2.4.2",
    "files": [
      { "file": "path", "lines": 12, "chars": 345, "words": 7, "size": 1234, "mtime": "RFC3339", "ext": "rs" }
    ],
    "summary": { "lines": 12, "chars": 345, "words": 7, "files": 1 },
    "by": [
      { "label": "By Extension", "rows": [ { "key": "rs", "lines": 12, "chars": 345, "count": 1 } ] }
    ]
  }
  ```
* `words` と `mtime` は条件により省略されます。`by` は指定がない場合 `null`。

### YAML

* JSON と同じスキーマ。ルートに `version` を含みます。

### JSONL

* `type = "file"` の行がファイルごとに出力され、末尾に `type = "total"` が 1 行
* 末尾 `total` 行には `version` を含みます
  例：

  ```json
  {"type":"file","file":"a.rs","lines":12,"chars":345,"mtime":"...","ext":"rs"}
  ...
  {"type":"total","version":"2.4.2","lines":123,"chars":4567,"words":7,"files":10}
  ```

## 集計（--by）の仕様

* `--by ext`:

  * 拡張子空（なし）は `"(noext)"` キー
* `--by dir[=N]`:

  * パスの親ディレクトリを先頭から N 階層（既定 1）。直下ファイルは `"."`
  * 例：`a/b/c.rs`、N=2 → `a/b`
* `--by mtime[:gran]`:

  * `gran` は `day`, `week`, `month`。出力ラベルは `By Mtime (gran)`
  * 例：`2025-10-27`, `2025-W44`, `2025-10` などのキー
  * **mtime が無いファイルは `"(no mtime)"` として集計**

## サイズ指定（--max_size/--min_size）

* サフィックス：`K`, `KB`, `KiB`, `M`, `MB`, `MiB`, `G`, `GB`, `GiB`, `T`, `TB`, `TiB`
* `_` は無視されます（例: `1_024KiB`）

## 挙動のコツと注意

* パフォーマンスと精度：

  * 既定は高速テキスト判定（先頭 1024B の NUL 検出）。誤判定が問題になる場合は `--fast_text_detect=false` を推奨
* 既定除外ディレクトリは `--no_default_prune` で解除可能
* パス表示を短くしたい場合は `--trim_root <REPO_ROOT>` を活用
* 既定ソートは `lines:desc`。旧バージョンの昇順に合わせたい場合は `--sort lines` を明示

## 例

1. 一覧トップ 20件、Markdown で保存
   `count_lines --top 20 --format md > result.md`

2. 拡張子 rs と py のみ、単語数込み、比率列
   `count_lines --ext rs,py --words --ratio`

3. 直近1か月に更新、dir=2 と month で集計（各上位5行）
   `count_lines --mtime_since 2025-09-27 --by dir=2 --by mtime:month --by_limit 5`

4. `.gitignore` 尊重、隠し/リンク除外、表は出さず集計と合計のみ
   `count_lines --git --summary_only --by ext`

5. サイズ 10KiB〜1MiB、行数 50〜500、式で RS のみ
   `count_lines --min_size 10KiB --max_size 1MiB --min_lines 50 --max_lines 500 --filter "ext == 'rs'"`

6. JSON スナップショット → 差分比較

   ```bash
   count_lines --format json > old.json
   # 作業…
   count_lines --format json > new.json
   count_lines --compare old.json new.json
   ```

## 終了コード

* 0: 成功
* 非0: 失敗（`--strict` が有効なら 1件失敗でも非0）

## ライセンス・作者

* ライセンス: プロジェクトのライセンスに従う（`Cargo.toml` / リポジトリ参照）
* 作者: jungamer-64