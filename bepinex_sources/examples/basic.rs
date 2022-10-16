use bepinex_sources::{github::GitHubApi, models::bleeding_edge::bepinex::BepInExRelease};
use semver::Version;

fn main() -> anyhow::Result<()> {
    let min_ver = Version::parse("5.4.21").unwrap();
    let mut gh = GitHubApi::new("BepInEx", "BepInEx");
    gh.set_pre_releases(true);
    gh.set_min_tag(Some(min_ver));

    let releases = gh.get_all()?;
    let bix_releases: Vec<BepInExRelease> = releases.into_iter().map(|r| r.into()).collect();

    println!("{bix_releases:#?}");
    Ok(())
}
