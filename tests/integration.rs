#![allow(dead_code)]
//! Integration test suite for end-to-end scenarios.

#[path = "e2e/end_to_end.rs"]
mod end_to_end;
#[path = "e2e/output_formats.rs"]
mod output_formats;
#[path = "e2e/snapshot_comparison.rs"]
mod snapshot_comparison;
