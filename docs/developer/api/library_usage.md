# Library Usage

`count_lines` exposes its core functionality as a reusable library crate.

```rust
use count_lines_core::{run_from_cli, Args};

fn main() -> anyhow::Result<()> {
    run_from_cli()
}
```

See `crates/core/src/lib.rs` for the public API surface.
