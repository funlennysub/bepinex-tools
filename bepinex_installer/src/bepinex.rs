use std::{fmt::Display, sync::Arc};

use octocrab::Octocrab;
use semver::Version;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BepInExRelease {
    pub version: Version,
    pub assets: Vec<BepInExAsset>,
    pub flavor: ReleaseFlavor,
}

impl Display for BepInExRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BepInExAsset {
    pub name: String,
    pub url: String,
}

impl BepInEx {
    pub async fn get_stable_releases(
        oc: Arc<Octocrab>,
    ) -> Result<Vec<BepInExRelease>, octocrab::Error> {
        let repo_releases = oc.repos("BepInEx", "BepInEx");
        let res_list = repo_releases.releases();

        let mut releases: Vec<BepInExRelease> = Vec::new();
        let mut page = 1u32;
        loop {
            let mut fetched = res_list.list().page(page).per_page(100).send().await?.items;
            if fetched.is_empty() {
                break;
            }

            fetched.retain(|r| semver::Version::parse(&r.tag_name.replace('v', "")).is_ok());
            releases.extend(fetched.into_iter().map(|r| {
                BepInExRelease {
                    version: semver::Version::parse(&r.tag_name.replace('v', "")).unwrap(),
                    assets: r
                        .assets
                        .into_iter()
                        .map(|a| BepInExAsset {
                            name: a.name,
                            url: a.url.to_string(),
                        })
                        .collect(),
                    flavor: ReleaseFlavor::Stable,
                }
            }));
            page += 1;
        }

        Ok(releases)
    }

    // TODO
    pub fn _be_release_to_bix_release() {}
}
