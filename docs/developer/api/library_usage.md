# Library Usage

`count_lines` exposes its CLI helpers alongside the reusable core library.

```rust
use count_lines::{run_from_cli, Args};

fn main() -> anyhow::Result<()> {
    run_from_cli()
}
```

See `src/lib.rs` and `crates/core/src/lib.rs` for the public API surface.
