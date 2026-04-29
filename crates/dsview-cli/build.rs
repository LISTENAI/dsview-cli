use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=DSVIEW_BUILD_VERSION");

    if let Some(workspace_root) = workspace_root() {
        emit_git_rerun_hints(&workspace_root);
    }

    let version = env::var("DSVIEW_BUILD_VERSION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(detect_git_tag_version)
        .unwrap_or_else(|| env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION should be set"));

    println!("cargo:rustc-env=DSVIEW_BUILD_VERSION={version}");
}

fn workspace_root() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").ok()?);
    manifest_dir.parent()?.parent().map(Path::to_path_buf)
}

fn emit_git_rerun_hints(workspace_root: &Path) {
    let git_metadata = workspace_root.join(".git");
    println!("cargo:rerun-if-changed={}", git_metadata.display());

    if git_metadata.is_dir() {
        emit_git_dir_rerun_hints(&git_metadata);
        return;
    }

    if git_metadata.is_file() {
        if let Ok(contents) = fs::read_to_string(&git_metadata) {
            if let Some(git_dir) = parse_gitdir_path(&git_metadata, &contents) {
                emit_git_dir_rerun_hints(&git_dir);
            }
        }
    }
}

fn emit_git_dir_rerun_hints(git_dir: &Path) {
    let head_path = git_dir.join("HEAD");
    println!("cargo:rerun-if-changed={}", head_path.display());

    let refs_path = git_dir.join("refs");
    println!("cargo:rerun-if-changed={}", refs_path.display());

    let packed_refs_path = git_dir.join("packed-refs");
    println!("cargo:rerun-if-changed={}", packed_refs_path.display());

    if let Ok(head_contents) = fs::read_to_string(&head_path) {
        if let Some(reference) = head_contents.strip_prefix("ref: ") {
            println!(
                "cargo:rerun-if-changed={}",
                git_dir.join(reference.trim()).display()
            );
        }
    }
}

fn parse_gitdir_path(git_metadata: &Path, contents: &str) -> Option<PathBuf> {
    let git_dir = contents.strip_prefix("gitdir: ")?.trim();
    let git_dir_path = Path::new(git_dir);

    if git_dir_path.is_absolute() {
        Some(git_dir_path.to_path_buf())
    } else {
        git_metadata
            .parent()
            .map(|parent| parent.join(git_dir_path))
    }
}

fn detect_git_tag_version() -> Option<String> {
    let workspace_root = workspace_root()?;

    git_output(&workspace_root, &["describe", "--tags", "--exact-match"])
}

fn git_output(workspace_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
