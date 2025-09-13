use anyhow::Context;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn shallow_clone_to(repo: &str, branch: &str, dest: &Path) -> anyhow::Result<(String, String)> {
    println!("🌱cloning {repo} @ {branch} -> {dest:?}");
    if dest.exists() {
        std::fs::remove_dir_all(dest).context("cleaning dest before clone")?;
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, repo])
        .arg(dest)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("running git clone")?;
    if !status.success() {
        anyhow::bail!("git clone failed");
    }

    // Capture the commit hash locally before removing .git
    let output = Command::new("git")
        .current_dir(dest)
        .args(["rev-parse", "HEAD"])
        .stdin(Stdio::null())
        .output()
        .context("running git rev-parse HEAD")?;
    if !output.status.success() {
        anyhow::bail!("git rev-parse failed");
    }
    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Capture the author date/time in strict ISO 8601 (RFC 3339) format
    let output_time = Command::new("git")
        .current_dir(dest)
        .args(["show", "-s", "--format=%aI", "HEAD"]) // author date, ISO-8601 strict
        .stdin(Stdio::null())
        .output()
        .context("running git show for author date")?;
    if !output_time.status.success() {
        anyhow::bail!("git show failed");
    }
    let commit_time_iso = String::from_utf8_lossy(&output_time.stdout)
        .trim()
        .to_string();

    // Remove .git
    let git_dir = dest.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir).context("removing .git")?;
    }
    Ok((commit_hash, commit_time_iso))
}
