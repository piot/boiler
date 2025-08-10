use anyhow::{Context, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;
use walkdir::WalkDir;

fn norm_component(s: &str) -> &str {
    s.trim_matches('/').trim()
}

/// Copy only selected subfolders from `src_root` into `dest_root`.
pub fn copy_selected_dirs(
    src_root: &Path,
    dest_root: &Path,
    picks: &[PathBuf],
) -> anyhow::Result<()> {
    if picks.is_empty() {
        return copy_dir_recursive(src_root, dest_root);
    }
    for pick in picks {
        let pick_norm = norm_component(pick.to_str().unwrap());
        if pick_norm.is_empty() {
            continue;
        }
        let src_dir = src_root.join(pick_norm);
        if !src_dir.exists() {
            warn!(?pick_norm, ?src_root, "include root not found",);
            continue;
        }
        copy_dir_recursive(&src_dir, &dest_root.join(pick_norm))?;
    }
    Ok(())
}

/// Copy a directory tree: src_dir -> dest_dir (dest_dir becomes/contains the contents of src_dir)
pub fn copy_dir_recursive(src_dir: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    println!("📂copying directory {src_dir:?} -> {dest_dir:?}");
    if !src_dir.exists() {
        return Err(anyhow!("missing: {}", src_dir.display()));
    }
    for entry in WalkDir::new(src_dir) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let rel = entry.path().strip_prefix(src_dir).unwrap();
        let out_path = dest_dir.join(rel);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &out_path).with_context(|| {
            format!("copy {} -> {}", entry.path().display(), out_path.display())
        })?;
    }
    Ok(())
}
