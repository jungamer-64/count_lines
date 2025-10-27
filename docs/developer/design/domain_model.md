# Domain Model Overview

This document describes the key entities and value objects that make up the
`count_lines` domain:

- **FileMeta** – immutable metadata about an input file.
- **FileEntry** – a discovered file alongside its metadata.
- **FileStats** – calculated statistics for a single file.
- **Summary** – aggregated totals across all processed files.

Refer to `crates/core/src/domain/model/` for the corresponding implementations.
