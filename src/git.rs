use anyhow::Context;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn shallow_clone_to(repo: &str, branch: &str, dest: &Path) -> anyhow::Result<()> {
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

    // Remove .git
    let git_dir = dest.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir).context("removing .git")?;
    }
    Ok(())
}
