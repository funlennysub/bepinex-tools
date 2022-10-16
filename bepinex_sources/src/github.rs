use reqwest::header::{self, HeaderMap};
use semver::Version;

use crate::models::github::releases::GitHubRelease;

pub struct GitHubApi {
    owner: String,
    repo: String,
    pre_releases: bool,
    min_tag: Option<Version>,
}

impl GitHubApi {
    pub fn new(owner: &str, repo: &str) -> Self {
        GitHubApi {
            owner: owner.into(),
            repo: repo.into(),
            pre_releases: false,
            min_tag: None,
        }
    }

    pub fn set_pre_releases(&mut self, pre_releases: bool) -> &mut Self {
        self.pre_releases = pre_releases;
        self
    }

    pub fn set_min_tag(&mut self, min_tag: Option<Version>) -> &mut Self {
        self.min_tag = min_tag;
        self
    }

    pub fn filter_release(&self, release: &GitHubRelease) -> bool {
        // if pres enabled, return all releases
        if !self.pre_releases && release.pre_release {
            return false;
        }

        match &self.min_tag {
            Some(tag) => release.tag_name >= *tag,
            None => true,
        }
    }

    pub fn get_releases(&self, per_page: u32, page: u32) -> anyhow::Result<Vec<GitHubRelease>> {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page={}&page={}",
            self.repo, self.owner, per_page, page
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "rust-reqwest/gh-api".parse().expect("Invalid UA"),
        );

        let resp = reqwest::blocking::Client::new()
            .get(&api_url)
            .headers(headers)
            .send()?;
        let releases = resp.json::<Vec<GitHubRelease>>()?;

        Ok(releases)
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<GitHubRelease>> {
        let mut releases = Vec::new();
        let mut page = 1;

        loop {
            let fetched = self.get_releases(100, page)?;
            if fetched.is_empty() {
                break;
            }

            releases.extend(fetched);
            page += 1;
        }

        Ok(releases
            .into_iter()
            .filter(|rel| self.filter_release(rel))
            .collect())
    }
}
