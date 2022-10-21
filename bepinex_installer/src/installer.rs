use std::time::Duration;

use bepinex_helpers::game::{Game, GameType};
use bepinex_sources::{
    bepinex::{AssetDownloader, BepInEx, BepInExRelease, ReleaseFlavor},
    version::VersionExt,
};
use eframe::{
    egui::{CentralPanel, ComboBox, Direction, FontFamily::Proportional, FontId, TextStyle, Ui},
    App,
};
use egui_extras::{Size, StripBuilder};
use egui_toast::{ToastOptions, Toasts};

use crate::MIN_IL2CPP_STABLE_VERSION;

#[derive(Default)]
pub struct Installer {
    pub release_flavor: ReleaseFlavor,
    pub bepinex: BepInEx,
    pub selected_bie: Option<BepInExRelease>,
    pub games: Vec<Game>,
    pub selected_game: Option<Game>,
    pub dl_promise: Option<poll_promise::Promise<anyhow::Result<()>>>,
    pub shown_toast: bool,
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

    fn show_bie_select(self: &mut Installer, ui: &mut Ui) {
        ComboBox::from_id_source("bie_selector")
            .width(ui.available_width() - 8.0)
            .selected_text(
                &self
                    .selected_bie
                    .as_ref()
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "None".to_string()),
            )
            .show_ui(ui, |ui| {
                for bie_ver in self
                    .bepinex
                    .releases
                    .iter()
                    .filter(|r| r.flavor == self.release_flavor)
                {
                    ui.selectable_value(
                        &mut self.selected_bie,
                        Some(bie_ver.to_owned()),
                        &bie_ver.version.to_string(),
                    );
                }
            });
    }

    fn release_flavor_select(self: &mut Installer, ui: &mut Ui) {
        ui.radio_value(
            &mut self.release_flavor,
            ReleaseFlavor::Stable,
            ReleaseFlavor::Stable.to_string(),
        );
        ui.radio_value(
            &mut self.release_flavor,
            ReleaseFlavor::BleedingEdge,
            ReleaseFlavor::BleedingEdge.to_string(),
        );
    }

    fn install_bie(self: &mut Installer, toasts: &mut Toasts, options: ToastOptions) {
        if let (Some(selected_game), Some(selected_bie)) = (&self.selected_game, &self.selected_bie)
        {
            let supported_ver = selected_game.ty == Some(GameType::UnityIL2CPP)
                && selected_bie.version >= *MIN_IL2CPP_STABLE_VERSION;

            let supported = supported_ver || (selected_game.ty != Some(GameType::UnityIL2CPP));
            if !supported {
                toasts.error(
                    format!(
                        "Minimal BepInEx for this game is {}",
                        *MIN_IL2CPP_STABLE_VERSION
                    ),
                    options,
                );
                return;
            }

            let query = selected_bie.to_query(selected_game);
            if let Some(asset) = selected_bie.select_asset(query) {
                let game = selected_game.clone();
                self.dl_promise = Some(poll_promise::Promise::spawn_thread("dl", move || {
                    asset.download(&game)
                }));
            } else {
                toasts.error("Failed to find asset", options);
            }
        }
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
                .size(Size::exact(30.0))
                .size(Size::exact(30.0))
                .size(Size::exact(30.0))
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| ui.label("Unity game"));
                            });
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| self.show_games_select(ui));
                            });
                        });
                    });
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| ui.label("Version"));
                            });
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| self.show_bie_select(ui));
                            });
                        });
                    });
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| ui.label("Release type"));
                            });
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| {
                                    self.release_flavor_select(ui);
                                });
                            });
                        });
                    });
                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(40.0))
                            .vertical(|mut strip| {
                                if let (Some(selected_game), Some(_selected_bie)) =
                                    (&self.selected_game, &self.selected_bie)
                                {
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
                                                    Some(ver) => ui.monospace(ver.display()),
                                                    None => ui.monospace("None"),
                                                }
                                            });
                                        });
                                    });
                                    strip.cell(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            let options = ToastOptions {
                                                show_icon: true,
                                                ..ToastOptions::with_duration(Duration::from_secs(
                                                    2,
                                                ))
                                            };
                                            if ui.button("Install").clicked() {
                                                self.shown_toast = false;
                                                self.install_bie(&mut toasts, options);
                                            }
                                            if let Some(dl_promise) = &self.dl_promise {
                                                if let Some(r) = dl_promise.ready() {
                                                    if let Err(e) = r {
                                                        toasts.error(e.to_string(), options);
                                                    } else {
                                                        toasts.success("Installed.", options);
                                                    }
                                                    self.dl_promise = None;
                                                } else if !self.shown_toast {
                                                    toasts.info("Downloading...", options);
                                                    self.shown_toast = true;
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
        ctx.request_repaint();
    }
}
