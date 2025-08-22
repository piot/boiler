use crate::github::GitHubShortName;
use crate::{DepotId, SteamAppId};
use seq_map::SeqMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::info;

#[derive(Debug)]
pub struct BinariesPlatform {
    pub depot: DepotId,
}

#[derive(Debug)]
pub struct BinariesYini {
    pub repo: GitHubShortName,
    pub name: String, // release name
    pub version: String,
    pub macos: BinariesPlatform,
    pub windows: BinariesPlatform,
    pub linux: BinariesPlatform,
}

#[derive(Debug)]
pub struct ContentYini {
    pub depot: DepotId,
    pub repo: GitHubShortName,
    pub copy: Vec<(String, PathBuf)>,
}

#[derive(Debug)]
pub struct BoilerYini {
    pub app_id: SteamAppId,
    pub binaries: BinariesYini,
    pub content: ContentYini,
}

impl Default for BoilerYini {
    fn default() -> Self {
        Self {
            app_id: 0,
            binaries: BinariesYini {
                repo: GitHubShortName {
                    org: "".to_string(),
                    name: "".to_string(),
                },
                name: "".to_string(),
                version: "".to_string(),
                macos: BinariesPlatform { depot: 0 },
                windows: BinariesPlatform { depot: 0 },
                linux: BinariesPlatform { depot: 0 },
            },
            content: ContentYini {
                depot: 0,
                repo: GitHubShortName {
                    org: "".to_string(),
                    name: "".to_string(),
                },
                copy: Vec::new(),
            },
        }
    }
}

pub fn parse_yini(yini_path: &Path) -> Option<BoilerYini> {
    info!(?yini_path, "⚙️reading the lovely yini file");

    let mut ini = BoilerYini::default();

    let Ok(str) = fs::read_to_string(yini_path) else {
        return None;
    };

    let mut parser = yini::Parser::new(&str);

    let root = parser.parse();

    ini.app_id = root.get("steam_app_id").unwrap().as_int().unwrap() as SteamAppId;

    let binaries_root = root.get("binaries").unwrap().as_object().unwrap();

    {
        let repo = binaries_root
            .get("repo")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        ini.binaries.repo = GitHubShortName::from_str(&repo).unwrap();
        ini.binaries.name = binaries_root
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        ini.binaries.version = binaries_root
            .get("version")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        let windows_root = binaries_root.get("windows").unwrap().as_object().unwrap();
        ini.binaries.windows.depot =
            windows_root.get("depot").unwrap().as_int().unwrap() as DepotId;

        let linux_root = binaries_root.get("linux").unwrap().as_object().unwrap();
        ini.binaries.linux.depot = linux_root.get("depot").unwrap().as_int().unwrap() as DepotId;

        let macos_root = binaries_root.get("macos").unwrap().as_object().unwrap();
        ini.binaries.macos.depot = macos_root.get("depot").unwrap().as_int().unwrap() as DepotId;
    }

    let content_root = root.get("content").unwrap().as_object().unwrap();
    {
        ini.content.depot = content_root.get("depot").unwrap().as_int().unwrap() as DepotId;

        let repo = content_root
            .get("repo")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        ini.content.repo = GitHubShortName::from_str(&repo).unwrap();

        let mut converted = Vec::new();

        for v in content_root.get("copy").unwrap().as_array().unwrap() {
            let (key, value) = v.as_tuple().unwrap();
            converted.push((
                key.as_str().unwrap().to_string(),
                Path::new(&value.as_str().unwrap()).to_path_buf(),
            ));
        }

        ini.content.copy = converted;
    }

    Some(ini)
}
