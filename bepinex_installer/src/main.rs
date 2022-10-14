#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod bepinex;
pub mod installer;

use bepinex::BepInEx;
use bepinex_helpers::game::get_unity_games;
use eframe::{run_native, NativeOptions};
use lazy_static::lazy_static;
use semver::Version;

use crate::installer::Installer;

lazy_static! {
    pub static ref MIN_SUPPORTED_STABLE_VERSION: Version = Version::parse("5.4.11").unwrap();
    pub static ref MIN_IL2CPP_STABLE_VERSION: Version = Version::parse("6.0.0-pre.1").unwrap();
}
pub const OLDEST_SUPPORTED_BE: u16 = 510;

#[tokio::main]
async fn main() {
    let octocrab = octocrab::instance();

    let stable_releases = BepInEx::get_stable_releases(octocrab).await;
    if stable_releases.is_err() {
        return;
    }
    let mut stable_releases = stable_releases.unwrap();

    stable_releases.retain(|x| x.version >= *MIN_SUPPORTED_STABLE_VERSION);

    let games = get_unity_games();
    if games.is_err() {
        return;
    }
    let mut games = games.unwrap();
    games.sort();

    let options = NativeOptions {
        follow_system_theme: true,
        transparent: false,
        resizable: false,
        initial_window_size: Some(eframe::egui::vec2(300.0, 450.0)),
        ..NativeOptions::default()
    };

    let installer = Installer {
        bepinex: BepInEx {
            releases: stable_releases,
        },
        games,
        ..Installer::default()
    };

    run_native(
        "BepInEx Installer",
        options,
        Box::new(|_cc| Box::new(installer)),
    )
}
