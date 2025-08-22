use anyhow::{Context, anyhow};
use glob::glob;
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

fn has_glob_meta(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[') || s.contains('{')
}

#[derive(Debug, Clone)]
pub struct CopyMapping {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Copy mappings where each entry specifies a source path (file/dir/glob) relative to `src_root`
/// and a destination directory relative to `dest_root`.
///
/// Examples:
///   "data/something.p*" -> "new_data/"
///   "data/just_dir/" -> "another_data/"
///   "data/just_dir/another*.txt" -> "another_data/"
#[allow(dead_code)]
pub fn copy_mappings(
    src_root: &Path,
    dest_root: &Path,
    mappings: &[CopyMapping],
) -> anyhow::Result<()> {
    for mapping in mappings {
        let from_norm = norm_component(mapping.from.to_str().unwrap());
        let to_norm = norm_component(mapping.to.to_str().unwrap());
        if from_norm.is_empty() {
            continue;
        }
        let abs_from = src_root.join(from_norm);
        let abs_to_root = dest_root.join(to_norm);

        // Ensure destination directory exists
        fs::create_dir_all(&abs_to_root)?;

        if has_glob_meta(from_norm) {
            let pattern = abs_from.to_string_lossy().to_string();
            let entries = glob(&pattern).with_context(|| format!("invalid glob: {pattern}"))?;
            let mut matched_any = false;
            for entry in entries {
                matched_any = true;
                let path = entry?;
                if path.is_dir() {
                    // Copy all files inside the matched directory into `to`, dropping the top-level dir name
                    for e in WalkDir::new(&path) {
                        let e = e?;
                        if e.file_type().is_dir() {
                            continue;
                        }
                        let rel = e.path().strip_prefix(&path).unwrap();
                        let out = abs_to_root.join(rel);
                        if let Some(parent) = out.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(e.path(), &out).with_context(|| {
                            format!("copy {} -> {}", e.path().display(), out.display())
                        })?;
                    }
                } else if path.is_file() {
                    let base = path.parent().unwrap();
                    let rel = path.strip_prefix(base).unwrap();
                    let out = abs_to_root.join(rel);
                    if let Some(parent) = out.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&path, &out)
                        .with_context(|| format!("copy {} -> {}", path.display(), out.display()))?;
                }
            }
            if !matched_any {
                warn!(?from_norm, ?src_root, "no matches for glob");
            }
            continue;
        }

        if abs_from.is_dir() {
            // Copy all files inside the directory into `to`
            for e in WalkDir::new(&abs_from) {
                let e = e?;
                if e.file_type().is_dir() {
                    continue;
                }
                let rel = e.path().strip_prefix(&abs_from).unwrap();
                let out = abs_to_root.join(rel);
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(e.path(), &out)
                    .with_context(|| format!("copy {} -> {}", e.path().display(), out.display()))?;
            }
            continue;
        }

        if abs_from.is_file() {
            let file_name = abs_from.file_name().unwrap();
            let out = abs_to_root.join(file_name);
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&abs_from, &out)
                .with_context(|| format!("copy {} -> {}", abs_from.display(), out.display()))?;
            continue;
        }

        warn!(?from_norm, ?src_root, "path not found");
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
