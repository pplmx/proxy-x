//! Build script for proxy-x.
//!
//! Installs Git hooks from `.husky/` using the husky-rs library API.
//! This mirrors what husky-rs's own build script does, but is invoked
//! from proxy-x's build so that husky-rs can remain a build-dependency
//! (not a runtime dependency).

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use husky_rs::{should_skip_installation, HUSKY_DIR};

fn main() {
    println!("cargo:rerun-if-env-changed=NO_HUSKY_HOOKS");

    if should_skip_installation() {
        return;
    }

    let project_root = find_project_root();
    let hooks_dir = project_root.join(HUSKY_DIR);

    // Re-run build script when .husky/ changes
    println!("cargo:rerun-if-changed={}", hooks_dir.display());

    if !hooks_dir.exists() {
        return;
    }

    // Set core.hooksPath to .husky
    let status = Command::new("git")
        .args(["config", "core.hooksPath", HUSKY_DIR])
        .current_dir(&project_root)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=proxy-x: Configured core.hooksPath to .husky");
        }
        Ok(_) => {
            eprintln!("proxy-x: Failed to configure core.hooksPath via git config");
        }
        Err(e) => {
            eprintln!(
                "proxy-x: Git command not found, skipping hook config: {}",
                e
            );
        }
    }

    // Ensure hook files are executable (Unix-like systems only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(entries) = fs::read_dir(&hooks_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        let mut perms = metadata.permissions();
                        if perms.mode() & 0o111 == 0 {
                            perms.set_mode(perms.mode() | 0o111);
                            let _ = fs::set_permissions(&path, perms);
                        }
                    }
                }
            }
        }
    }
}

/// Find the project root by searching for `.git` starting from OUT_DIR.
fn find_project_root() -> PathBuf {
    let start_dir = env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("Failed to get current directory"));

    start_dir
        .ancestors()
        .find(|path| {
            let git_entry = path.join(".git");
            git_entry.is_dir() || git_entry.is_file()
        })
        .map(|p| p.to_path_buf())
        .unwrap_or(start_dir)
}
