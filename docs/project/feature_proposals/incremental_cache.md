# Incremental Cache & Watch Mode

## Summary
- Introduce a file-statistics cache and change-notification loop to avoid rescanning entire workspaces after small edits.
- Primary goals: speed up local feedback during development, reduce CI cost for PR diff checks, and leverage existing JSON diff tooling.
- Scope covers a single execution (`--incremental`) and a long-running watcher (`--watch`); defaults remain backward compatible.
- Status: incremental cache + watch mode shipped (`--incremental`, `--cache-dir`, `--watch`, `--watch-interval`, `--watch-output`, `--cache-verify`, `--clear-cache`)。`--watch-output jsonl` は各リフレッシュ結果を JSON Lines で通知し、`changed_files` / `removed_files` を含みます。

## CLI Additions (Draft)
- `--incremental` &ndash; reuse cached measurements and recompute only for changed files.
- `--watch` / `-w` &ndash; monitor paths and re-run incremental aggregation on change.
- `--cache-dir <path>` &ndash; override cache location (defaults to platform cache dir or `.cache/count_lines`).
- `--cache-verify` &ndash; fall back to fast metadata checks, with optional hash verification for higher fidelity.
- `--clear-cache` &ndash; wipe cache for the target workspace.
- `--watch-interval <secs>` &ndash; polling interval for environments without native notify support.
- `--watch-output {jsonl|full}` &ndash; control emission style while watching (`jsonl` for events, `full` for complete reports).

## Cache Design
- Format: serde JSON persisted at `<cache_dir>/count_lines-cache-<root-hash>.json`.
- Entries keyed by canonical file path; fields capture size, mtime, aggregated metrics, and optional hash.
- Writes use temp-file + atomic rename; corrupted caches trigger rebuild with a warning.
- Cache metadata includes versioning to allow migrations.

## Change Detection
- Default: size + mtime act as the file ETag.
- Optional `--cache-verify`: compute fast hashes (e.g., xxhash) to guard against timestamp skew.
- On execution:
  - New files → measure and insert.
  - Modified files → re-measure.
  - Deleted files → drop from cache and totals.
  - Moves/renames handled via canonical-path keying.

## Watch Mode
- Implement with `notify` crate for native file events, with polling fallback controlled by `--watch-interval`.
- Debounce bursts to limit recomputations.
- Output modes:
  - `full`: rerun normal presenters after each change.
  - `jsonl`: emit compact change events suitable for streaming/automation.

## Error Handling
- Unreadable files: log warnings, continue unless `--strict`.
- Cache I/O errors: notify user, rebuild cache.
- In-flight edits: double-check pattern (stat before/after read); retry limited times.

## Dependencies
- `notify` (>= 5) for cross-platform watching.
- `dirs` (if not already present) for platform cache roots.
- `xxhash-rust` or similar for optional verification hashes.

## Testing Strategy
- Unit: cache read/write round trips, metadata vs hash-based invalidation, deletion handling.
- Integration:
  - Run full scan → mutate file → `--incremental` returns updated totals.
  - Watch loop observes create/update/delete events in a temp workspace.
- Performance: benchmark small-change scenarios to confirm expected speedups (target 3×–10×).

## Delivery Plan (Phased)
1. **PR-1**: Core cache module, CLI flags for `--incremental` / `--cache-dir`, incremental execution path, initial tests.
2. **PR-2**: Watcher abstraction, CLI `--watch`, debounce logic, integration coverage.
3. **PR-3**: Extended options (`--cache-verify`, `--clear-cache`, `--watch-output`), documentation, CI wiring, benchmarks.

## Risks & Mitigations
- Notify platform differences → provide polling fallback.
- Cache bloat in huge repos → keep metrics on disk; explore binary/compressed formats later.
- Precision vs speed trade-off → hash verification opt-in.

## Acceptance Criteria
- `--incremental` results match full scans across the existing suite.
- Watch mode reliably tracks create/update/delete events and reissues output.
- Atomic cache writes with auto-recovery on corruption.
- Documentation updated (usage, changelog, help) and CI passes (fmt, clippy, tests).
