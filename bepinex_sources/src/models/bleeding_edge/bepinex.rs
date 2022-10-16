use std::{collections::HashMap, fmt::Display, io::Cursor};

use bepinex_helpers::game::Game;
use semver::Version;
use zip::ZipArchive;

use crate::models::github::releases::GitHubRelease;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BepInEx {
    pub releases: Vec<BepInExRelease>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseFlavor {
    Stable,
    BleedingEdge,
}

impl Default for ReleaseFlavor {
    fn default() -> Self {
        Self::Stable
    }
}

pub type BepInExAssets = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BepInExRelease {
    pub version: Version,
    pub assets: BepInExAssets,
    pub flavor: ReleaseFlavor,
}

impl From<GitHubRelease> for BepInExRelease {
    fn from(res: GitHubRelease) -> Self {
        Self {
            version: res.tag_name,
            assets: res
                .assets
                .into_iter()
                .map(|asset| (asset.name, asset.browser_download_url))
                .collect(),
            flavor: ReleaseFlavor::Stable,
        }
    }
}

impl Display for BepInExRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

pub trait AssetDownloader {
    fn download(&self, query: String, game: &Game) -> anyhow::Result<()>;
}

impl AssetDownloader for BepInExAssets {
    fn download(&self, query: String, game: &Game) -> anyhow::Result<()> {
        let asset = self.get(&query).expect("Asset not found");

        let client = reqwest::blocking::Client::new();
        let resp = client.get(asset).send()?.bytes()?;

        let content = Cursor::new(resp.to_vec());
        ZipArchive::new(content)?.extract(&game.path)?;

        Ok(())
    }
}
