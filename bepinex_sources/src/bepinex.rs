use std::{fmt::Display, io::Cursor};

use bepinex_helpers::{game::Game, game_type};
use semver::Version;
use zip::ZipArchive;

use crate::{
    models::{bleeding_edge::builds::BuildsRelease, github::releases::GitHubRelease},
    version::VersionExt,
};

#[derive(Debug, Default, Clone)]
pub struct BepInEx {
    pub releases: Vec<BepInExRelease>,
}

impl BepInEx {
    pub fn latest(&self) -> Option<BepInExRelease> {
        self.releases
            .iter()
            .filter(|r| r.flavor == ReleaseFlavor::Stable)
            .map(|r| r.to_owned())
            .collect::<Vec<_>>()
            .first()
            .map(|r| r.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseFlavor {
    Stable,
    BleedingEdge,
}

impl Display for ReleaseFlavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseFlavor::Stable => write!(f, "Stable"),
            ReleaseFlavor::BleedingEdge => write!(f, "Bleeding edge"),
        }
    }
}

impl Default for ReleaseFlavor {
    fn default() -> Self {
        Self::Stable
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BepInExAsset {
    pub name: String,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BepInExRelease {
    pub version: Version,
    pub assets: Vec<BepInExAsset>,
    pub flavor: ReleaseFlavor,
}

impl BepInExRelease {
    pub fn select_asset(&self, query: String) -> Option<BepInExAsset> {
        self.assets
            .iter()
            .find(|a| a.name == query)
            .map(|a| a.to_owned())
    }

    pub fn to_query(&self, game: &Game) -> String {
        match self.flavor {
            ReleaseFlavor::Stable => match self.version.major {
                6 => format!(
                    "BepInEx_{}_{}_{}.zip",
                    game_type!(game.ty),
                    game.arch,
                    &self.version
                ),
                _ => format!("BepInEx_{}_{}.0.zip", game.arch, self),
            },
            ReleaseFlavor::BleedingEdge => {
                let artifact_id = self
                    .version
                    .pre
                    .split('.')
                    .filter_map(|e| e.parse::<u32>().ok())
                    .collect::<Vec<_>>()[0];
                match artifact_id >= 600 {
                    true => format!(
                        "BepInEx-{}-win-{}-{}.zip",
                        game.ty.as_ref().unwrap(),
                        game.arch,
                        self.version,
                    ),
                    false => {
                        format!(
                            "BepInEx_{}_{}_{}_{}.zip",
                            game_type!(game.ty),
                            game.arch,
                            self.version.build,
                            self.version.mmpp()
                        )
                    }
                }
            }
        }
    }
}

impl From<GitHubRelease> for BepInExRelease {
    fn from(rel: GitHubRelease) -> Self {
        Self {
            version: rel.tag_name,
            assets: rel
                .assets
                .into_iter()
                .map(|r| BepInExAsset {
                    name: r.name,
                    link: r.browser_download_url,
                })
                .collect(),
            flavor: ReleaseFlavor::Stable,
        }
    }
}

impl From<BuildsRelease> for BepInExRelease {
    fn from(rel: BuildsRelease) -> Self {
        Self {
            version: rel.version,
            assets: rel
                .assets
                .into_iter()
                .map(|r| BepInExAsset {
                    name: r.name,
                    link: r.link,
                })
                .collect(),
            flavor: ReleaseFlavor::BleedingEdge,
        }
    }
}

impl Display for BepInExRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

pub trait AssetDownloader {
    fn download(&self, game: &Game) -> anyhow::Result<()>;
}

impl AssetDownloader for BepInExAsset {
    fn download(&self, game: &Game) -> anyhow::Result<()> {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(&self.link).send()?.bytes()?;

        let content = Cursor::new(resp.to_vec());
        ZipArchive::new(content)?.extract(&game.path)?;

        Ok(())
    }
}
