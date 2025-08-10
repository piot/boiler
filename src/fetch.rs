use crate::download::download_to_with_name;
use crate::extract::extract_archive;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Download URL to temp file (supports optional GitHub token), then extract to `dest_root`.
pub fn extract_to_target(url: &str, token: Option<&str>, dest_root: &Path) -> anyhow::Result<()> {
    println!("📡fetching {}", url);
    let dir = tempdir()?;
    let archive_path = download_to_with_name(dir.path(), url, token)?;
    println!("🗜️extracting -> {}", dest_root.display());
    extract_archive(&archive_path, dest_root)?;
    flatten_single_top_dir(dest_root)?;
    Ok(())
}

/// HACK: If there's exactly one top-level directory, move its contents up one level (to dest_root).
fn flatten_single_top_dir(dest_root: &Path) -> anyhow::Result<()> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dest_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "__MACOSX" {
            continue;
        }
        entries.push(entry);
    }

    if entries.len() != 1 {
        return Ok(());
    }

    let top = &entries[0];

    if !top.file_type()?.is_dir() {
        return Ok(());
    }

    let top_path = top.path();

    for entry in fs::read_dir(&top_path)? {
        let e = entry?;
        fs::rename(e.path(), dest_root.join(e.file_name()))?;
    }

    fs::remove_dir_all(top_path)?;

    Ok(())
}
