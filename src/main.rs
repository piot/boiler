mod args;
mod download;
mod extract;
mod fetch;
mod fsutil;
mod git;
mod github;
mod vdf;
mod yini;

use crate::args::{parse, Target};
use crate::fetch::extract_to_target;
use crate::fsutil::{CopyMapping, copy_dir_recursive, copy_mappings, clean_dir};
use crate::git::shallow_clone_to;
use crate::github::{github_download_url, github_repo_url};
use crate::yini::parse_yini;
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
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

    // Refuse unsafe/public branches
    if let Some(ref live_branch_value) = args.live_branch {
        let branch_lower = live_branch_value.to_lowercase();
        if branch_lower == "default" || branch_lower == "public" {
            return Err(anyhow!(
                "refusing to use forbidden Steam branch '{}' (use a non-public branch)",
                live_branch_value
            ));
        }
    }

    let maybe_init = parse_yini(&args.ini);

    let Some(ini) = maybe_init else {
        return Err(anyhow!("couldn't parse ini {:?}", args.ini));
    };

    println!(
        "🧯cooldown: cleaning {} and temp_dir {:?}",
        if args.keep_build_dir { "temp_dir only" } else { "build_dir and temp_dir" },
        args.temp_dir
    );
    if !args.keep_build_dir {
        if args.build_dir.exists() {
            fs::remove_dir_all(&args.build_dir).context("removing build dir")?;
        }
    }
    if args.temp_dir.exists() {
        fs::remove_dir_all(&args.temp_dir).context("removing temp dir")?;
    }

    // resolve targets
    let selected_targets: Vec<Target> = if args.targets.is_empty() {
        vec![Target::Content, Target::Mac, Target::Linux, Target::Windows]
    } else {
        args.targets.clone()
    };
    let process_content = selected_targets.contains(&Target::Content);
    let process_mac = selected_targets.contains(&Target::Mac);
    let process_linux = selected_targets.contains(&Target::Linux);
    let process_windows = selected_targets.contains(&Target::Windows);

    // create/clean dirs as needed
    if process_content {
        let p = args.build_dir.join("data");
        if args.keep_build_dir { clean_dir(&p)?; } else { fs::create_dir_all(&p)?; }
    }
    if process_mac {
        let p = args.build_dir.join("binaries/macos");
        if args.keep_build_dir { clean_dir(&p)?; } else { fs::create_dir_all(&p)?; }
    }
    if process_windows {
        let p = args.build_dir.join("binaries/windows");
        if args.keep_build_dir { clean_dir(&p)?; } else { fs::create_dir_all(&p)?; }
    }
    if process_linux {
        let p = args.build_dir.join("binaries/linux");
        if args.keep_build_dir { clean_dir(&p)?; } else { fs::create_dir_all(&p)?; }
    }

    let temp_shared_root = args.temp_dir.join("data");
    if process_content {
        fs::create_dir_all(&temp_shared_root)?;
    }

    // Start downloads and building
    let mut content_commit_hash: Option<String> = None;
    let mut content_commit_time_iso: Option<String> = None;
    if process_content {
        println!("🦄fetching your lovely game content...");
        let repo = github_repo_url(&ini.content.repo);
        let (hash, time_iso) = shallow_clone_to(&repo, "main", &temp_shared_root)?;
        content_commit_hash = Some(hash);
        content_commit_time_iso = Some(time_iso);

        println!("🍬grabbing the goodies...");

        let mut mappings = Vec::new();
        for (source, target_sub_dir) in ini.content.copy {
            mappings.push(CopyMapping {
                from: source.parse()?,
                to: target_sub_dir,
            })
        }

        copy_mappings(&temp_shared_root, &args.build_dir, &mappings)?;
    }

    println!("🛳️finding binaries to ship...");
    let prefix = github_download_url(
        &ini.binaries.repo,
        &ini.binaries.version,
        &ini.binaries.name,
    );

    let mut macos_target: Option<std::path::PathBuf> = None;
    let mut windows_target: Option<std::path::PathBuf> = None;
    let mut linux_target: Option<std::path::PathBuf> = None;

    if process_mac {
        let mac_arm_url = format!("{prefix}-darwin-arm64.tar.gz");
        let target = args.build_dir.join("binaries/macos");
        extract_to_target(&mac_arm_url, args.github_token.as_deref(), &target)?;
        copy_dir_recursive(&args.steam_redist.join("osx"), &*target)?;
        macos_target = Some(target);
    }

    if process_windows {
        let windows_url = format!("{prefix}-windows-x86_64.zip");
        let target = args.build_dir.join("binaries/windows");
        extract_to_target(&windows_url, args.github_token.as_deref(), &target)?;
        copy_dir_recursive(&args.steam_redist.join("win64"), &target)?;
        windows_target = Some(target);
    }

    if process_linux {
        let linux_url = format!("{prefix}-linux-x86_64.tar.gz");
        let target = args.build_dir.join("binaries/linux");
        extract_to_target(&linux_url, args.github_token.as_deref(), &target)?;
        copy_dir_recursive(&args.steam_redist.join("linux64"), &target)?;
        linux_target = Some(target);
    }

    let vdf_dir = args.build_dir.clone();

    println!("🧱writing those pesky .vdf files...");
    let depots: Vec<Depot> = vec![
        Depot { id: ini.content.depot, vdf: "depot_content.vdf".to_string() },
        Depot { id: ini.binaries.macos.depot, vdf: "depot_macos.vdf".to_string() },
        Depot { id: ini.binaries.linux.depot, vdf: "depot_linux.vdf".to_string() },
        Depot { id: ini.binaries.windows.depot, vdf: "depot_windows.vdf".to_string() },
    ];

    let app_build_vdf_file = vdf_dir.join(format!("app_build_{}.vdf", ini.app_id));
    let root_vdf_contents = vdf::app_build(
        ini.app_id,
        "Internal build",
        args.live_branch.as_deref(),
        &depots,
    );
    println!("  ✅ {app_build_vdf_file:?}");
    fs::write(&app_build_vdf_file, root_vdf_contents)?;

    if process_content {
        let content_vdf = vdf::depot(ini.content.depot, &args.build_dir.join("data"));
        let content_vdf_file = vdf_dir.join("depot_content.vdf");
        println!("  ✅ {content_vdf_file:?}");
        fs::write(&content_vdf_file, content_vdf)?;
    }

    if process_mac {
        let macos_vdf = vdf::depot_with_os_filter(
            ini.binaries.macos.depot,
            &args.build_dir.join("binaries/macos"),
            "macos",
        );
        let macos_vdf_file = vdf_dir.join("depot_macos.vdf");
        println!("  ✅ {macos_vdf_file:?}");
        fs::write(macos_vdf_file, macos_vdf)?;
    }

    if process_linux {
        let linux_vdf = vdf::depot_with_os_filter(
            ini.binaries.linux.depot,
            &args.build_dir.join("binaries/linux"),
            "linux",
        );
        let linux_vdf_file = vdf_dir.join("depot_linux.vdf");
        println!("  ✅ {linux_vdf_file:?}");
        fs::write(linux_vdf_file, linux_vdf)?;
    }

    if process_windows {
        let windows_vdf = vdf::depot_with_os_filter(
            ini.binaries.windows.depot,
            &args.build_dir.join("binaries/windows"),
            "windows",
        );
        let windows_vdf_file = vdf_dir.join("depot_windows.vdf");
        println!("  ✅ {windows_vdf_file:?}");
        fs::write(windows_vdf_file, windows_vdf)?;
    }

    let borrow = app_build_vdf_file.canonicalize().unwrap();
    let complete_app_build_vdf_path = borrow.to_str().unwrap();

    let now_utc = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    // Content buildinfo in data/
    println!("🏗 writing buildinfo files...");
    if process_content {
        let content_buildinfo_path = args.build_dir.join("data").join("buildinfo_content.txt");
        let content_buildinfo = format!(
            "repo: {}/{}\ncommit: {}\ncommitted_at_utc: {}\nbuilt_at_utc: {}\n",
            ini.content.repo.org,
            ini.content.repo.name,
            content_commit_hash.as_deref().unwrap_or(""),
            content_commit_time_iso.as_deref().unwrap_or(""),
            now_utc
        );
        fs::write(&content_buildinfo_path, content_buildinfo)?;
    }

    // Binaries buildinfo in each selected platform directory
    let bin_buildinfo = format!(
        "repo: {}/{}\nversion: {}\nbuilt_at_utc: {}\n",
        ini.binaries.repo.org, ini.binaries.repo.name, ini.binaries.version, now_utc
    );
    if let Some(target) = &macos_target {
        fs::write(target.join("buildinfo_binaries.txt"), &bin_buildinfo)?;
    }
    if let Some(target) = &windows_target {
        fs::write(target.join("buildinfo_binaries.txt"), &bin_buildinfo)?;
    }
    if let Some(target) = &linux_target {
        fs::write(target.join("buildinfo_binaries.txt"), &bin_buildinfo)?;
    }

    println!("🎉 all steamed up!");

    println!(
        r#"
tip:

brew install steamcmd # or install from https://developer.valvesoftware.com/wiki/SteamCMD

verify the files in the build/ directory and then upload using:

steamcmd +login [your_steam_email] +run_app_build {complete_app_build_vdf_path:?} +quit

optionally set the branch to go live:

steamcmd +login [your_steam_email] +run_app_build {complete_app_build_vdf_path:?} +setlive internal +quit
"#
    );

    Ok(())
}
