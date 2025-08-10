use anyhow::{Context, anyhow};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_DISPOSITION;
use std::path::{Path, PathBuf};
use std::{fs, io};

fn filename_from_headers_or_url(resp: &reqwest::blocking::Response, url: &str) -> PathBuf {
    if let Some(cd) = resp.headers().get(CONTENT_DISPOSITION) {
        if let Ok(s) = cd.to_str() {
            // very light parse
            if let Some(name) = s.split("filename=").nth(1) {
                let trimmed = name.trim_matches('"').trim();
                if !trimmed.is_empty() {
                    return PathBuf::from(trimmed);
                }
            }
        }
    }

    let parsed = reqwest::Url::parse(url).ok();
    if let Some(seg) = parsed.and_then(|u| u.path_segments()?.last().map(|s| s.to_string())) {
        return PathBuf::from(seg);
    }

    PathBuf::from("download.bin")
}

pub fn download_to_with_name(
    tmp_dir: &Path,
    url: &str,
    token: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let client = Client::new();
    let mut req = client.get(url);
    if let Some(t) = token {
        req = req
            .bearer_auth(t)
            .header("Accept", "application/octet-stream");
    }
    let mut resp = req.send().context("downloading asset")?;
    if !resp.status().is_success() {
        return Err(anyhow!("download failed: {} {}", resp.status(), url));
    }
    let fname = filename_from_headers_or_url(&resp, url);
    let out_path = tmp_dir.join(fname);
    let mut out = fs::File::create(&out_path)?;
    io::copy(&mut resp, &mut out)?;
    Ok(out_path)
}
