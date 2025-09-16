use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "boiler")]
pub struct Args {
    /// Path to .boiler.yini configuration
    #[arg(value_name = "INI")]
    pub(crate) ini: PathBuf,

    /// Optional GitHub token for private assets (env GITHUB_TOKEN also supported)
    #[arg(long)]
    pub(crate) github_token: Option<String>,

    /// steam sdk
    #[arg(long, default_value = "steam_redist")]
    pub(crate) steam_redist: PathBuf,

    /// Staging root (default: ./build)
    #[arg(long, default_value = "build")]
    pub(crate) build_dir: PathBuf,

    /// temporary files (default: ./temp)
    #[arg(long, default_value = "temp")]
    pub(crate) temp_dir: PathBuf,

    /// Optional Steam setlive branch to set (omit to not set live)
    #[arg(long = "live-branch", aliases = ["setlive"], value_name = "STEAM_BRANCH")]
    pub(crate) live_branch: Option<String>,

    /// Leave build directory intact (do not delete at start)
    #[arg(long)]
    pub(crate) keep_build_dir: bool,

    /// Which parts to process. Comma-separated: linux, mac, windows, content
    /// Example: --targets mac,content
    #[arg(long, value_delimiter = ',', value_enum)]
    pub(crate) targets: Vec<Target>,
}

pub fn parse() -> Args {
    Args::parse()
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum Target {
    /// macOS binaries and depot (aliases: macos, osx)
    #[value(alias = "macos")]
    #[value(alias = "osx")]
    Mac,
    /// Linux binaries and depot
    Linux,
    /// Windows binaries and depot
    Windows,
    /// Game content (data/) and depot
    Content,
}
