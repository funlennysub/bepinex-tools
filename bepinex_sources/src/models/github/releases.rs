use std::fmt;

use semver::Version;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::version::VersionExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRelease {
    #[serde(rename = "prerelease")]
    pub pre_release: bool,
    #[serde(deserialize_with = "parse_tag")]
    pub tag_name: Version,
    pub assets: Vec<GitHubAsset>,
}

fn try_to_parse(version: &str) -> Version {
    // Greatest fix, until ErrorKind is hidden from external crates, can't really do much
    let ver: Vec<u64> = version
        .split('.')
        .map(|e| e.parse::<u64>().unwrap_or_default())
        .collect();

    Version::parse(version).unwrap_or_else(|_| Version::fix_version(ver[0], ver[1]))
}

fn parse_tag<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    struct ReleaseTagVisitor;
    impl<'de> Visitor<'de> for ReleaseTagVisitor {
        type Value = Version;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
            formatter.write_str("release tag")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let stripped = v.replacen('v', "", 1);
            Ok(try_to_parse(&stripped))
        }
    }
    deserializer.deserialize_str(ReleaseTagVisitor)
}
