#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod installer;

use bepinex_helpers::game::get_unity_games;
use bepinex_sources::{
    github::GitHubApi,
    models::bleeding_edge::bepinex::{BepInEx, BepInExRelease},
};
use eframe::{egui, run_native, NativeOptions};
use lazy_static::lazy_static;
use semver::Version;

use crate::installer::Installer;

lazy_static! {
    pub static ref MIN_SUPPORTED_STABLE_VERSION: Version = Version::parse("5.4.11").unwrap();
    pub static ref MIN_IL2CPP_STABLE_VERSION: Version = Version::parse("6.0.0-pre.1").unwrap();
    pub static ref MIN_SUPPORTED_BE_VERSION: Version = Version::parse("6.0.0-be.510").unwrap();
}

fn main() {
    let mut gh = GitHubApi::new("BepInEx", "BepInEx");
    gh.set_pre_releases(true);
    gh.set_min_tag(Some(MIN_SUPPORTED_STABLE_VERSION.clone()));

    let stable_releases = gh.get_all().unwrap_or_default();

    let releases: Vec<BepInExRelease> = stable_releases.into_iter().map(|r| r.into()).collect();

    let games = get_unity_games();
    if games.is_err() {
        return;
    }
    let mut games = games.unwrap();
    games.sort();

    let min_size = Some(egui::vec2(300.0, 450.0));
    let options = NativeOptions {
        follow_system_theme: true,
        transparent: false,
        initial_window_size: min_size,
        min_window_size: min_size,
        ..NativeOptions::default()
    };

    let installer = Installer {
        bepinex: BepInEx { releases },
        games,
        ..Installer::default()
    };

    run_native(
        "BepInEx Installer",
        options,
        Box::new(|_cc| Box::new(installer)),
    )
}
