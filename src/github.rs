use std::str::FromStr;

#[derive(Debug)]
pub struct GitHubShortName {
    pub org: String,
    pub name: String,
}

impl FromStr for GitHubShortName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments: Vec<_> = s.split("/").collect();
        assert_eq!(segments.len(), 2, "should be two");

        Ok(Self {
            org: segments[0].to_string(),
            name: segments[1].to_string(),
        })
    }
}

pub fn github_repo_url(repo: GitHubShortName) -> String {
    format!("https://github.com/{}/{}.git", repo.org, repo.name)
}

pub fn github_download_url(repo: GitHubShortName, version: &str, package_name: &str) -> String {
    format!(
        "https://github.com/{}/{}/releases/download/v{version}/{}",
        repo.org, repo.name, package_name
    )
}
