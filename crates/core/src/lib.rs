#![no_std]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::semicolon_if_nothing_returned)]
extern crate alloc;

pub mod config;
pub mod language;
pub mod parser;
pub mod stats;
