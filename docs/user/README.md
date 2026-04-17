# count_lines

`count_lines` は、ディレクトリ配下のファイルを走査して `lines/chars` を計測する CLI ツールです。
必要に応じて `words` / `sloc` を追加し、`table/csv/tsv/json/yaml/md/jsonl` で出力できます。

## できること

- 並列走査（`--jobs`）
- include/exclude/ext などの実用フィルタ
- サイズ・行数・文字数・更新時刻による絞り込み
- `--watch` による再計測
- `--compare old.json new.json` によるスナップショット差分

## インストール

```bash
cargo install --git https://github.com/jungamer-64/count_lines
```

またはリポジトリで:

```bash
cargo install --path crates/cli
```

## クイックスタート

```bash
# 全体を計測
count_lines .

# 拡張子を絞る
count_lines --ext rs,toml --sort lines:desc .

# JSONへ保存
count_lines --format json . > snapshot.json

# 差分比較
count_lines --compare old.json new.json
```

## よく使うオプション

- 出力: `--format`, `--sort`, `--total-row`, `--count-newlines-in-chars`
- フィルタ: `--include`, `--exclude`, `--ext`, `--min/max-size`, `--min/max-lines`, `--min/max-chars`, `--words`, `--sloc`, `--min/max-words`, `--mtime-since/until`, `--map-ext`
- 走査: `--hidden`, `--follow`, `--no-gitignore`, `--jobs`, `--max-depth`, `--walk-threads`
- モード: `--strict`, `--watch`, `--watch-interval`, `--watch-output`
- 比較: `--compare <OLD> <NEW>`

## 詳細ドキュメント

- CLI 完全リファレンス: `docs/user/CLI_REFERENCE.md`
- 使い方ガイド: `docs/user/USAGE.md`
- サンプル: `docs/user/examples/basic_usage.sh`, `docs/user/examples/advanced_filtering.sh`

## License

MIT または Apache-2.0 のデュアルライセンスです。
