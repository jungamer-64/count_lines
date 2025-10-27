# Ports & Adapters

Adapters live under `crates/core/src/infrastructure/` and implement the ports
defined by the application layer. This keeps the core logic insulated from I/O
details and makes it straightforward to plug in new adapters for different
targets.
