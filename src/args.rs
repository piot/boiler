use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "boiler")]
pub struct Args {
    #[arg(long, default_value = "")]
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
}

pub fn parse() -> Args {
    Args::parse()
}
