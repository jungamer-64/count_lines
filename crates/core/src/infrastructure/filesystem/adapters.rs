mod git;
mod inputs;
mod matcher;

pub(crate) use git::collect_git_files;
pub(crate) use inputs::{read_files_from_lines, read_files_from_null};
pub(crate) use matcher::{PathMatcher, should_process_entry};
