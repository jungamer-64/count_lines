// src/domain/compute.rs
mod aggregate;
mod process;
mod sort;

pub use aggregate::{AggregationGroup, Aggregator};
pub use process::process_entries;
pub use sort::apply_sort;
