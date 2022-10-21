use semver::{BuildMetadata, Prerelease, Version};

pub trait VersionExt {
    fn mmp(&self) -> String;
    fn mmpp(&self) -> String;
    fn display(&self) -> String;
    fn fix_version(major: u64, minor: u64) -> Version;
}

impl VersionExt for Version {
    fn mmp(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    fn mmpp(&self) -> String {
        format!("{}.{}.{}-{}", self.major, self.minor, self.patch, self.pre)
    }

    fn display(&self) -> String {
        match self.pre.is_empty() {
            true => self.mmp(),
            false => self.mmpp(),
        }
    }

    fn fix_version(major: u64, minor: u64) -> Version {
        Version {
            major,
            minor,
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }
}
