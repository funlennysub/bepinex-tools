use bepinex_sources::{bepinex::BepInExRelease, builds::BuildsApi};

fn main() -> anyhow::Result<()> {
    let mut builds = BuildsApi::new("https://builds.bepinex.dev");
    builds.set_min_build_id(Some(657));
    let b = builds.get_builds()?;
    let bie_releases: Vec<BepInExRelease> = b.into_iter().map(|r| r.into()).collect();

    println!("{bie_releases:#?}");
    Ok(())
}
