use std::path::PathBuf;

use crate::{
    domain::config::Config,
    error::{InfrastructureError, Result},
};

pub(crate) fn collect_git_files(config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in &config.paths {
        let output = std::process::Command::new("git")
            .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "."])
            .current_dir(root.clone())
            .output()
            .map_err(|source| InfrastructureError::FileSystemOperation {
                operation: "git ls-files".to_string(),
                path: root.clone(),
                source,
            })?;
        if !output.status.success() {
            let details = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(
                InfrastructureError::GitError { operation: "git ls-files".to_string(), details }.into()
            );
        }
        for chunk in output.stdout.split(|&b| b == 0) {
            if let Some(path_str) = parse_git_output_chunk(chunk) {
                files.push(root.join(path_str));
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn parse_git_output_chunk(chunk: &[u8]) -> Option<String> {
    if chunk.is_empty() {
        return None;
    }
    let s = String::from_utf8_lossy(chunk).trim().to_string();
    (!s.is_empty()).then_some(s)
}
