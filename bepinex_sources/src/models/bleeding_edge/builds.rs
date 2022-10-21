use semver::Version;

#[derive(Debug)]
pub struct BuildsAsset {
    pub name: String,
    pub link: String,
}

#[derive(Debug)]
pub struct BuildsRelease {
    pub artifact_id: usize,
    pub version: Version,
    pub assets: Vec<BuildsAsset>,
}
