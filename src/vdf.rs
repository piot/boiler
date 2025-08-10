use crate::{Depot, DepotId, SteamAppId};
use std::path::Path;

pub fn app_build(
    app_id: SteamAppId,
    description: &str,
    live_branch: &str,
    depots: &[Depot],
) -> String {
    let mut depots_string = String::new();
    for depot in depots {
        depots_string += &format!("            \"{}\" \"{}\"\n", depot.id, depot.vdf);
    }

    let vdf = format!(
        r#""AppBuild"
{{
    "appid"    "{app_id}"
    "desc"     "{description}"
    "setlive"  "{live_branch}"

    "depots"
    {{
{depots_string}
    }}
}}"#
    );

    vdf
}

pub fn depot(depot_id: DepotId, content_root: &Path) -> String {
    let absolute_path = content_root
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let vdf = format!(
        r#""DepotBuildConfig"
{{
    "depotid"     "{depot_id}"
    "contentroot" "{absolute_path}" //  must be absolute canonical full path
    "filemapping" {{
        "LocalPath" "*"
        "DepotPath" "."
        "recursive" "1"
    }}
}}"#
    );

    vdf
}

pub fn depot_with_os_filter(depot_id: DepotId, content_root: &Path, os_name: &str) -> String {
    let absolute_path = content_root
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let vdf = format!(
        r#""DepotBuildConfig"
{{
    "depotid"     "{depot_id}"
    "contentroot" "{absolute_path}" //  must be absolute canonical full path
    "filemapping" {{
        "LocalPath" "*"
        "DepotPath" "."
        "recursive" "1"
    }}
    "config" {{ 
        "oslist" "{os_name}" 
    }}
}}"#
    );

    vdf
}
