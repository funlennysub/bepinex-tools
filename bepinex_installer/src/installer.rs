use std::time::Duration;

use bepinex_helpers::game::{Game, GameType};
use bepinex_sources::bepinex::{AssetDownloader, BepInEx, BepInExRelease, ReleaseFlavor};
use eframe::{
    egui::{
        Button, CentralPanel, ComboBox, Direction, FontFamily::Proportional, FontId, RichText,
        TextStyle, Ui,
    },
    App,
};
use egui_extras::{Size, StripBuilder};
use egui_toast::{ToastOptions, Toasts};

use crate::MIN_IL2CPP_STABLE_VERSION;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancedSettings {
    picker: bool,
    bleeding_edge: bool,
}

impl Installer {
    fn show_games_select(self: &mut Installer, ui: &mut Ui) {
        let size = 45.;
        ui.style_mut().text_styles = [(TextStyle::Button, FontId::new(size, Proportional))].into();

        ComboBox::from_id_source("game_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(RichText::new(
                self.selected_game
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Select a game".to_owned()),
            ))
            .show_ui(ui, |ui| {
                for game in self.games.iter() {
                    ui.selectable_value(&mut self.selected_game, Some(game.to_owned()), &game.name);
                }
            });
    }

    fn show_bix_select(self: &mut Installer, ui: &mut Ui) {
        let size = 45.;
        ui.style_mut().text_styles = [(TextStyle::Button, FontId::new(size, Proportional))].into();

        ComboBox::from_id_source("bix_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(
                self.selected_bix
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Select BepInEx version".to_owned()),
            )
            .show_ui(ui, |ui| {
                for bix_ver in self.bepinex.releases.iter().filter(|r| {
                    // .eq() because cargo fmt would move '== self' on a new line which makes it look ugly 🙄
                    r.flavor.eq(self
                        .advanced_settings
                        .map(|s| match s.bleeding_edge {
                            true => &ReleaseFlavor::BleedingEdge,
                            false => &ReleaseFlavor::Stable,
                        })
                        .unwrap_or(&ReleaseFlavor::Stable))
                }) {
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
        let mut toasts = Toasts::new()
            .anchor((10., 10.))
            .direction(Direction::TopDown)
            .align_to_end(false);
        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(50.0))
                .size(Size::exact(50.0))
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

                                    let supported = supported_ver
                                        || (selected_game.ty != Some(GameType::UnityIL2CPP));

                                    strip.cell(|ui| {
                                        ui.style_mut().text_styles = [
                                            (TextStyle::Body, FontId::new(20.0, Proportional)),
                                            (TextStyle::Monospace, FontId::new(18.0, Proportional)),
                                        ]
                                        .into();
                                        ui.group(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label("Game type:");
                                                match &selected_game.ty {
                                                    Some(ty) => ui.monospace(ty.to_string()),
                                                    None => ui.monospace("Not Mono or IL2CPP"),
                                                }
                                            });
                                            ui.separator();
                                            ui.horizontal(|ui| {
                                                ui.label("Installed BepInEx:");
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
                                            if ui.add(install_btn).clicked() {
                                                let options = ToastOptions {
                                                    show_icon: true,
                                                    ..ToastOptions::with_duration(
                                                        Duration::from_secs(5),
                                                    )
                                                };

                                                if !supported {
                                                    toasts.error(format!("Minimal BepInEx for this game is {}", *MIN_IL2CPP_STABLE_VERSION), options);
                                                    return;
                                                }

                                                let query =
                                                    selected_game.to_query(&selected_bix.version);
                                                let res = selected_bix
                                                    .assets
                                                    .download(query, selected_game);
                                                match res {
                                                    Ok(_) => {
                                                        toasts.success(
                                                            "Start the game so you can install mods.",
                                                            options,
                                                        );
                                                    }
                                                    Err(e) => {
                                                        toasts.error(e.to_string(), options);
                                                    }
                                                }
                                            }
                                        });
                                    })
                                }
                            });
                    })
                });
        });

        toasts.show(ctx);
    }
}
