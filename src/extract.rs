use anyhow::{Context, anyhow};
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

pub fn extract_archive(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dest).context("create dest dir")?;
    let name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name.ends_with(".zip") {
        let f = File::open(archive).context("open zip")?;
        let mut z = ZipArchive::new(f).context("read zip")?;
        z.extract(dest).context("extract zip")?;
        return Ok(());
    }

    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let f = File::open(archive).context("open tar.gz")?;
        let dec = GzDecoder::new(f);
        let mut ar = Archive::new(dec);
        ar.unpack(dest).context("extract tar.gz")?;
        return Ok(());
    }

    if name.ends_with(".tar") {
        let f = File::open(archive).context("open tar")?;
        let mut ar = Archive::new(f);
        ar.unpack(dest).context("extract tar")?;
        return Ok(());
    }

    Err(anyhow!(
        "unsupported archive type (expected .zip, .tar.gz/.tgz, or .tar): {}",
        archive.display()
    ))
}
