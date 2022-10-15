use bepinex_helpers::game::{Game, GameType};
use eframe::{
    egui::{Button, CentralPanel, ComboBox, Ui},
    App,
};
use egui_extras::{Size, StripBuilder};

use crate::{
    bepinex::{BepInEx, BepInExRelease},
    MIN_IL2CPP_STABLE_VERSION,
};

#[derive(Default, Debug)]
pub struct Installer {
    pub settings: bool,
    pub advanced_mode: bool,
    pub advanced_settings: Option<AdvancedSettings>,
    pub bepinex: BepInEx,
    pub selected_bix: Option<BepInExRelease>,
    pub games: Vec<Game>,
    pub selected_game: Option<Game>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedSettings {
    picker: bool,
    bleeding_edge: bool,
}

impl Installer {
    fn show_games_select(self: &mut Installer, ui: &mut Ui) {
        ComboBox::from_id_source("game_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(
                self.selected_game
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Select a game".to_owned()),
            )
            .show_ui(ui, |ui| {
                for game in self.games.iter() {
                    ui.selectable_value(&mut self.selected_game, Some(game.to_owned()), &game.name);
                }
            });
    }

    fn show_bix_select(self: &mut Installer, ui: &mut Ui) {
        ComboBox::from_id_source("bix_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(
                self.selected_bix
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Select BepInEx version".to_owned()),
            )
            .show_ui(ui, |ui| {
                for bix_ver in self.bepinex.releases.iter() {
                    ui.selectable_value(
                        &mut self.selected_bix,
                        Some(bix_ver.to_owned()),
                        &bix_ver.version.to_string(),
                    );
                }
            });
    }
}

impl App for Installer {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(20.0))
                .size(Size::exact(20.0))
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.show_games_select(ui);
                    });
                    strip.cell(|ui| {
                        self.show_bix_select(ui);
                    });

                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(40.0))
                            .vertical(|mut strip| {
                                if let (Some(selected_game), Some(selected_bix)) =
                                    (&self.selected_game, &self.selected_bix)
                                {
                                    let supported_ver = selected_game.ty
                                        == Some(GameType::UnityIL2CPP)
                                        && selected_bix.version >= *MIN_IL2CPP_STABLE_VERSION;

                                    let enabled = supported_ver
                                        || (selected_game.ty != Some(GameType::UnityIL2CPP));

                                    strip.cell(|ui| {
                                        ui.group(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label("Game type: ");
                                                match &selected_game.ty {
                                                    Some(ty) => ui.monospace(ty.to_string()),
                                                    None => ui.monospace("Not Mono or IL2CPP"),
                                                }
                                            });
                                            ui.horizontal(|ui| {
                                                ui.label("Installed BepInEx: ");
                                                match &selected_game.bepinex_version {
                                                    Some(bix) => ui.monospace(bix.to_string()),
                                                    None => ui.monospace("None"),
                                                }
                                            });
                                        });
                                    });
                                    strip.cell(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            let install_btn = Button::new("Install").small();
                                            if ui.add_enabled(enabled, install_btn).clicked() {
                                                todo!(
                                                    "
                                                Implement install logic:
                                                    - Download correct zip (bix version, game type)
                                                        - Support file names from 5.4.11
                                                    - Unzip it
                                                "
                                                )
                                            }
                                        });
                                    })
                                }
                            });
                    })
                });
        });
    }
}
