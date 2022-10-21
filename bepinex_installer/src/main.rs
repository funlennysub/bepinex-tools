#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod installer;

use bepinex_helpers::game::get_unity_games;
use bepinex_sources::{
    bepinex::{BepInEx, BepInExRelease},
    builds::BuildsApi,
    github::GitHubApi,
};
use eframe::{egui, run_native, NativeOptions};
use lazy_static::lazy_static;
use semver::Version;

use crate::installer::Installer;

lazy_static! {
    pub static ref MIN_SUPPORTED_STABLE_VERSION: Version = Version::parse("5.4.11").unwrap();
    pub static ref MIN_IL2CPP_STABLE_VERSION: Version = Version::parse("6.0.0-pre.1").unwrap();
}

fn main() {
    // TODO: populate Installer while the app running instead.
    let mut gh = GitHubApi::new("BepInEx", "BepInEx");
    gh.set_pre_releases(true);
    gh.set_min_tag(Some(MIN_SUPPORTED_STABLE_VERSION.clone()));

    let stable_releases = gh.get_all().unwrap_or_default();

    let be = BuildsApi::new("https://builds.bepinex.dev");
    let be_builds = be.get_builds().unwrap_or_default();

    let mut releases: Vec<BepInExRelease> = Vec::new();
    releases.extend(stable_releases.into_iter().map(|r| r.into()));
    releases.extend(be_builds.into_iter().map(|r| r.into()));

    let games = get_unity_games();
    if games.is_err() {
        return;
    }
    let mut games = games.unwrap();
    games.sort();

    let min_size = Some(egui::vec2(400.0, 450.0));
    let options = NativeOptions {
        follow_system_theme: true,
        transparent: false,
        initial_window_size: min_size,
        min_window_size: min_size,
        ..NativeOptions::default()
    };

    let bepinex = BepInEx { releases };

    let installer = Installer {
        bepinex: bepinex.clone(),
        selected_bie: bepinex.latest(),
        games,
        ..Installer::default()
    };

    run_native(
        "BepInEx Installer",
        options,
        Box::new(|_cc| Box::new(installer)),
    )
}
