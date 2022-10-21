#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod installer;

use eframe::{egui, run_native, NativeOptions};
use lazy_static::lazy_static;
use semver::Version;

use crate::installer::Installer;

lazy_static! {
    pub static ref MIN_SUPPORTED_STABLE_VERSION: Version = Version::parse("5.4.11").unwrap();
    pub static ref MIN_IL2CPP_STABLE_VERSION: Version = Version::parse("6.0.0-pre.1").unwrap();
}

fn main() {
    let min_size = Some(egui::vec2(400.0, 450.0));
    let options = NativeOptions {
        follow_system_theme: true,
        transparent: false,
        initial_window_size: min_size,
        min_window_size: min_size,
        ..NativeOptions::default()
    };

    run_native(
        "BepInEx Installer",
        options,
        Box::new(|_cc| Box::new(Installer::new())),
    )
}
