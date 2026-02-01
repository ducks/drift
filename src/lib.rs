use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Represents a single issue found during the drift audit.
#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    pub category: String,
    pub severity: String,
    pub message: String,
    pub path: Option<PathBuf>,
    pub line: Option<usize>,
}

/// Runs all drift audit checks and returns a list of found issues.
pub fn run_audit() -> Vec<Issue> {
    let mut issues = Vec::new();

    issues.extend(check_stale_configs());
    issues.extend(check_version_mismatches());
    issues.extend(check_dead_code_markers());
    issues.extend(check_git_drift());
    issues.extend(check_gitignore_drift());

    issues
}

/// Checks for stale configuration or backup files (e.g., .old, .bak).
fn check_stale_configs() -> Vec<Issue> {
    let mut issues = Vec::new();
    let stale_extensions = ["old", "bak", "tmp", "swp", "orig"];

    fn walk_dir(dir: &std::path::Path, extensions: &[&str], issues: &mut Vec<Issue>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if path
                        .file_name()
                        .is_some_and(|n| n != "target" && n != ".git")
                    {
                        walk_dir(&path, extensions, issues);
                    }
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.iter().any(|&s| ext == s) {
                            issues.push(Issue {
                                category: "stale_config".to_string(),
                                severity: "warning".to_string(),
                                message: "Stale configuration or backup file".to_string(),
                                path: Some(path),
                                line: None,
                            });
                        }
                    }
                }
            }
        }
    }

    walk_dir(std::path::Path::new("."), &stale_extensions, &mut issues);
    issues
}

/// Checks for version mismatches between toolchain files.
fn check_version_mismatches() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check rust-toolchain.toml vs Cargo.toml rust-version
    if std::path::Path::new("rust-toolchain.toml").exists() {
        if let Ok(content) = fs::read_to_string("rust-toolchain.toml") {
            if content.contains("nightly") {
                // Check if Cargo.toml has rust-version set (which conflicts with nightly)
                if let Ok(cargo) = fs::read_to_string("Cargo.toml") {
                    if cargo.contains("rust-version") {
                        issues.push(Issue {
                            category: "version_mismatch".to_string(),
                            severity: "warning".to_string(),
                            message: "rust-toolchain.toml uses nightly but Cargo.toml has rust-version set".to_string(),
                            path: Some(PathBuf::from("rust-toolchain.toml")),
                            line: None,
                        });
                    }
                }
            }
        }
    }

    // Check for .nvmrc vs package.json engines
    if std::path::Path::new(".nvmrc").exists() && std::path::Path::new("package.json").exists() {
        if let (Ok(nvmrc), Ok(pkg)) = (
            fs::read_to_string(".nvmrc"),
            fs::read_to_string("package.json"),
        ) {
            let nvmrc_version = nvmrc.trim();
            if !pkg.contains(nvmrc_version) && pkg.contains("\"engines\"") {
                issues.push(Issue {
                    category: "version_mismatch".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        ".nvmrc specifies {} but package.json engines may differ",
                        nvmrc_version
                    ),
                    path: Some(PathBuf::from(".nvmrc")),
                    line: None,
                });
            }
        }
    }

    issues
}

/// Searches for dead code markers like TODO, FIXME in source code.
fn check_dead_code_markers() -> Vec<Issue> {
    let mut issues = Vec::new();
    let markers = ["TODO", "FIXME", "XXX", "HACK"];

    fn scan_file(path: &std::path::Path, markers: &[&str], issues: &mut Vec<Issue>) {
        if let Ok(content) = fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                for marker in markers {
                    if line.contains(marker) {
                        issues.push(Issue {
                            category: "dead_code".to_string(),
                            severity: "info".to_string(),
                            message: format!("{} marker found", marker),
                            path: Some(path.to_path_buf()),
                            line: Some(line_num + 1),
                        });
                    }
                }
            }
        }
    }

    fn walk_source(dir: &std::path::Path, markers: &[&str], issues: &mut Vec<Issue>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if path
                        .file_name()
                        .is_some_and(|n| n != "target" && n != ".git" && n != "node_modules")
                    {
                        walk_source(&path, markers, issues);
                    }
                } else if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if matches!(
                        ext,
                        "rs" | "js" | "ts" | "py" | "go" | "java" | "c" | "cpp" | "h"
                    ) {
                        scan_file(&path, markers, issues);
                    }
                }
            }
        }
    }

    walk_source(std::path::Path::new("."), &markers, &mut issues);
    issues
}

/// Checks for uncommitted changes in git.
fn check_git_drift() -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("git").args(["status", "--porcelain"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if !lines.is_empty() {
            let modified = lines
                .iter()
                .filter(|l| l.starts_with(" M") || l.starts_with("M "))
                .count();
            let untracked = lines.iter().filter(|l| l.starts_with("??")).count();

            if modified > 0 {
                issues.push(Issue {
                    category: "git_drift".to_string(),
                    severity: "warning".to_string(),
                    message: format!("{} modified files not committed", modified),
                    path: None,
                    line: None,
                });
            }

            if untracked > 0 {
                issues.push(Issue {
                    category: "git_drift".to_string(),
                    severity: "info".to_string(),
                    message: format!("{} untracked files", untracked),
                    path: None,
                    line: None,
                });
            }
        }
    }

    issues
}

/// Checks for entries in .gitignore that don't match any files.
fn check_gitignore_drift() -> Vec<Issue> {
    let mut issues = Vec::new();

    match fs::read_to_string(".gitignore") {
        Ok(content) => {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Simple check: if it's a literal path (no wildcards) and doesn't exist
                if !line.contains('*') && !line.contains('?') {
                    let path = std::path::Path::new(line.trim_start_matches('/'));
                    if !path.exists() && !line.ends_with('/') {
                        // Skip common patterns that may not exist yet
                        if !matches!(
                            line,
                            "*.log"
                                | "*.tmp"
                                | ".env"
                                | ".env.local"
                                | "node_modules"
                                | "target"
                                | "dist"
                                | "build"
                        ) {
                            issues.push(Issue {
                                category: "gitignore_drift".to_string(),
                                severity: "info".to_string(),
                                message: format!(
                                    "Gitignore entry '{}' doesn't match any files",
                                    line
                                ),
                                path: Some(PathBuf::from(".gitignore")),
                                line: None,
                            });
                        }
                    }
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // No .gitignore file - nothing to check
        }
        Err(e) => {
            issues.push(Issue {
                category: "gitignore_drift".to_string(),
                severity: "warning".to_string(),
                message: format!("Failed to read .gitignore: {}", e),
                path: Some(PathBuf::from(".gitignore")),
                line: None,
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_audit_returns_vec() {
        let issues = run_audit();
        // Verify that any issues found are well-formed
        for issue in &issues {
            assert!(
                !issue.category.is_empty(),
                "Issue category should not be empty"
            );
            assert!(
                !issue.severity.is_empty(),
                "Issue severity should not be empty"
            );
            assert!(
                !issue.message.is_empty(),
                "Issue message should not be empty"
            );
        }
    }
}
