mod args;
mod download;
mod extract;
mod fetch;
mod fsutil;
mod git;
mod github;
mod vdf;
mod yini;

use crate::args::parse;
use crate::fetch::extract_to_target;
use crate::fsutil::{copy_dir_recursive, copy_selected_dirs};
use crate::git::shallow_clone_to;
use crate::github::{github_download_url, github_repo_url};
use crate::yini::parse_yini;
use anyhow::{Context, Result, anyhow};
use std::fs;
use tracing_subscriber::EnvFilter;

type DepotId = u64;
type SteamAppId = u64;

pub struct Depot {
    id: DepotId,
    vdf: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()) // respects RUST_LOG
        .init();

    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("🔥boiler {VERSION} - building up steam...");

    let mut args = parse();

    if args.github_token.is_none() {
        if let Ok(tok) = std::env::var("GITHUB_TOKEN") {
            if !tok.trim().is_empty() {
                args.github_token = Some(tok);
            }
        }
    }

    let maybe_init = parse_yini(&args.ini);

    let Some(ini) = maybe_init else {
        return Err(anyhow!("couldn't parse ini {:?}", args.ini));
    };

    println!(
        "🧯cooldown: cleaning build_dir {:?} and temp_dir {:?}",
        args.build_dir, args.temp_dir
    );
    if args.build_dir.exists() {
        fs::remove_dir_all(&args.build_dir).context("removing build dir")?;
    }
    if args.temp_dir.exists() {
        fs::remove_dir_all(&args.temp_dir).context("removing temp dir")?;
    }

    // create dirs
    fs::create_dir_all(args.build_dir.join("data"))?;
    fs::create_dir_all(args.build_dir.join("binaries/macos"))?;
    fs::create_dir_all(args.build_dir.join("binaries/windows"))?;
    fs::create_dir_all(args.build_dir.join("binaries/linux"))?;

    let temp_shared_root = args.temp_dir.join("data");
    fs::create_dir_all(&temp_shared_root)?;

    // Start downloads and building
    println!("🦄fetching your lovely game content...");
    let repo = github_repo_url(ini.content.repo);
    shallow_clone_to(&repo, "main", &temp_shared_root)?;

    println!("🍬grabbing the goodies...");
    let shared_dest = args.build_dir.join("data");
    copy_selected_dirs(&temp_shared_root, &shared_dest, &ini.content.directories)?;

    println!("🛳️finding binaries to ship...");
    let prefix = github_download_url(ini.binaries.repo, &ini.binaries.version, &ini.binaries.name);

    let mac_arm_url = format!("{prefix}-darwin-arm64.tar.gz");
    let macos_target = args.build_dir.join("binaries/macos");
    extract_to_target(&mac_arm_url, args.github_token.as_deref(), &macos_target)?;
    copy_dir_recursive(&args.steam_redist.join("osx"), &*macos_target)?;

    let windows_url = format!("{prefix}-windows-x86_64.zip");
    let windows_target = args.build_dir.join("binaries/windows");
    extract_to_target(&windows_url, args.github_token.as_deref(), &windows_target)?;
    copy_dir_recursive(&args.steam_redist.join("win64"), &windows_target)?;

    let linux_url = format!("{prefix}-linux-x86_64.tar.gz");
    let linux_target = args.build_dir.join("binaries/linux");
    extract_to_target(&linux_url, args.github_token.as_deref(), &linux_target)?;
    copy_dir_recursive(&args.steam_redist.join("linux64"), &linux_target)?;

    let vdf_dir = args.build_dir.clone();

    println!("🧱writing those pesky .vdf files...");
    let depots = [
        Depot {
            id: ini.content.depot,
            vdf: "depot_content.vdf".to_string(),
        },
        Depot {
            id: ini.binaries.macos.depot,
            vdf: "depot_macos.vdf".to_string(),
        },
        Depot {
            id: ini.binaries.linux.depot,
            vdf: "depot_linux.vdf".to_string(),
        },
        Depot {
            id: ini.binaries.windows.depot,
            vdf: "depot_windows.vdf".to_string(),
        },
    ];

    let app_build_vdf_file = vdf_dir.join(format!("app_build_{}.vdf", ini.app_id));
    let root_vdf_contents = vdf::app_build(ini.app_id, "Internal build", "internal", &depots);
    println!("  ✅ {app_build_vdf_file:?}");
    fs::write(&app_build_vdf_file, root_vdf_contents)?;

    let content_vdf = vdf::depot(depots[0].id, &args.build_dir.join("data"));
    let content_vdf_file = vdf_dir.join(depots[0].vdf.clone());
    println!("  ✅ {content_vdf_file:?}");
    fs::write(&content_vdf_file, content_vdf)?;

    let macos_vdf = vdf::depot_with_os_filter(
        depots[1].id,
        &args.build_dir.join("binaries/macos"),
        "macos",
    );
    let macos_vdf_file = vdf_dir.join(depots[1].vdf.clone());
    println!("  ✅ {macos_vdf_file:?}");
    fs::write(macos_vdf_file, macos_vdf)?;

    let linux_vdf = vdf::depot_with_os_filter(
        depots[2].id,
        &args.build_dir.join("binaries/linux"),
        "linux",
    );
    let linux_vdf_file = vdf_dir.join(depots[2].vdf.clone());
    println!("  ✅ {linux_vdf_file:?}");
    fs::write(linux_vdf_file, linux_vdf)?;

    let windows_vdf = vdf::depot_with_os_filter(
        depots[3].id,
        &args.build_dir.join("binaries/windows"),
        "windows",
    );

    let windows_vdf_file = vdf_dir.join(depots[3].vdf.clone());
    println!("  ✅ {windows_vdf_file:?}");
    fs::write(windows_vdf_file, windows_vdf)?;

    println!("🎉 all steamed up!");

    let borrow = app_build_vdf_file.canonicalize().unwrap();
    let complete_app_build_vdf_path = borrow.to_str().unwrap();
    println!(
        "tip:\n🖥️ brew install steamcmd\n\nverify the build directory and then upload:\n\n🖥️ steamcmd +login [your_steam_email] +run_app_build {complete_app_build_vdf_path:?} +quit\n\n"
    );

    Ok(())
}
