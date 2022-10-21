use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use semver::Version;

use crate::{
    models::bleeding_edge::builds::{BuildsAsset, BuildsRelease},
    s_parse, select,
};

lazy_static! {
    static ref VERISON_REGEX: Regex = Regex::new(
            r"((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)-(?:(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))))",
    ).unwrap();
}

pub struct BuildsApi {
    base_url: String,
    min_build_id: Option<usize>,
}

impl BuildsApi {
    pub fn new(base_url: &str) -> Self {
        BuildsApi {
            base_url: base_url.into(),
            min_build_id: None,
        }
    }

    pub fn set_base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }

    pub fn set_min_build_id(&mut self, min_build: Option<usize>) -> &mut Self {
        self.min_build_id = min_build;
        self
    }

    pub fn filter_builds(&self, build: &BuildsRelease) -> bool {
        match &self.min_build_id {
            Some(build_id) => build.artifact_id >= *build_id,
            None => true,
        }
    }

    pub fn get_builds(&self) -> anyhow::Result<Vec<BuildsRelease>> {
        let mut releases: Vec<BuildsRelease> = Vec::new();

        let resp = reqwest::blocking::Client::new()
            .get(format!("{}/projects/bepinex_be", &self.base_url))
            .send()?;
        let html = resp.text()?;
        let fragment = Html::parse_fragment(&html);

        let main_selector = s_parse!("main");
        let artifact_item_selector = s_parse!("div.artifact-item");
        let build_id_selector = s_parse!("span.artifact-id");
        let artifact_hash_selector = s_parse!("a.hash-button");

        let artifacts_list_selector = s_parse!("div.artifacts-list");
        let artifact_link_selector = s_parse!("a.artifact-link");

        let main = select!(fragment, &main_selector);
        for el in main.select(&artifact_item_selector) {
            let artifact_id = select!(el, &build_id_selector)
                .text()
                .filter_map(|e| e[1..].parse::<usize>().ok())
                .collect::<Vec<_>>()[0];
            let build_hash = select!(el, &artifact_hash_selector)
                .text()
                .collect::<Vec<_>>()[0]
                .to_string();
            let mut version: String = "".into();

            let mut assets: Vec<BuildsAsset> = Vec::new();
            let artifacts_list = select!(el, &artifacts_list_selector);
            for artifact_el in artifacts_list.select(&artifact_link_selector) {
                let download_link = artifact_el.value().attr("href").unwrap();

                let artifact_name = artifact_el.text().collect::<Vec<&str>>()[0].to_string();
                if version.is_empty() && let Some(version_m) = VERISON_REGEX.find(&artifact_name) {
                    version = format!("{}+{}", artifact_name[version_m.start()..version_m.end()].to_owned(), build_hash);
                }
                assets.push(BuildsAsset {
                    name: artifact_name,
                    link: format!("{}{}", self.base_url, download_link),
                });
            }
            releases.push(BuildsRelease {
                artifact_id,
                version: Version::parse(&version).unwrap(),
                assets,
            });
        }

        Ok(releases
            .into_iter()
            .filter(|b| self.filter_builds(b))
            .collect())
    }
}
