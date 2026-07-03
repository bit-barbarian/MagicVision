use egui::{CentralPanel, ComboBox, Panel, ScrollArea, TextureHandle};
use rfd::FileDialog;
use std::{io::Write, time::Duration};

use crate::{
    cache::card_cache::CachedCard,
    gui::{
        deck_file_formats::{DeckFileFormat, DeckList, FoilType, format_file_headers, format_line},
        draw_cards::{draw_deck_card, draw_match_card},
    },
    recognition::init::Engine,
    types::DynResult,
};

struct CardMatch {
    distance: u32,
    face_num: u8,
    card: CachedCard,
}

pub struct Application {
    engine: Engine,
    file_format: DeckFileFormat,
    current_texture: Option<TextureHandle>,
    current_matched_cards: Vec<CardMatch>,
    decklist: DeckList,
}

impl Default for Application {
    fn default() -> Self {
        let engine = Engine::start().expect("Unable to start recognition engine.");
        Self {
            engine,
            file_format: DeckFileFormat::Moxfield,
            current_texture: None,
            current_matched_cards: Vec::new(),
            decklist: DeckList { cards: Vec::new() },
        }
    }
}

impl Application {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
    fn save_deck(&self) -> DynResult<()> {
        let Some(outfile) = (match self.file_format {
            DeckFileFormat::Moxfield => FileDialog::new()
                .add_filter("Comma Separated Values", &["csv"])
                .save_file(),
            DeckFileFormat::Archidekt => FileDialog::new().add_filter("Text", &["txt"]).save_file(),
        }) else {
            return Ok(());
        };

        let mut outfile = std::fs::File::create(outfile)?;
        outfile.write_all(format_file_headers(&self.file_format).as_bytes())?;
        for dc in &self.decklist.cards {
            outfile.write_all(format_line(&self.file_format, dc).as_bytes())?;
        }

        Ok(())
    }

    fn draw_loading_ui(&mut self, ui: &mut egui::Ui) {
        egui_extras::install_image_loaders(ui);
        ui.centered_and_justified(|ui| {
            ui.heading("Loading...");
        });
    }

    fn draw_running_ui(&mut self, ui: &mut egui::Ui) {
        Panel::top("Top Bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Using");
                ComboBox::from_id_salt("Deck Source")
                    .selected_text(match self.file_format {
                        DeckFileFormat::Moxfield => "Moxfield",
                        DeckFileFormat::Archidekt => "Archidekt",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.file_format,
                            DeckFileFormat::Moxfield,
                            "Moxfield",
                        );
                        ui.selectable_value(
                            &mut self.file_format,
                            DeckFileFormat::Archidekt,
                            "Archidect",
                        );
                    });
                ui.label("file format.");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.heading("Magic Vision");
                });
            });
        });

        Panel::left("Camera Panel").show(ui, |ui| {
            ui.vertical(|ui| {
                if let Some(texture) = &self.current_texture {
                    ui.add(egui::Image::new(texture).shrink_to_fit());
                };

                if let Some(top_card) = self.current_matched_cards.first() {
                    ui.horizontal(|ui| {
                        ui.heading("Best Match:");
                    });
                    ui.heading(format!(
                        "{} [{} {}],",
                        &top_card.card.name,
                        &top_card.card.set.to_uppercase(),
                        &top_card.card.number,
                    ));
                    ui.heading(format!(
                        "{} face",
                        match top_card.face_num {
                            0 => "Front",
                            1 => "Back",
                            _ => "Other",
                        }
                    ));
                    ui.horizontal(|ui| {
                        ui.label(format!("(confidence: {})", &top_card.distance));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.menu_button("+", |ui| {
                                if ui.button("Add Nonfoil").clicked() {
                                    self.decklist.add(top_card.card.clone(), FoilType::None);
                                }
                                if ui.button("Add Foil").clicked() {
                                    self.decklist.add(top_card.card.clone(), FoilType::Foil);
                                }
                                if ui.button("Add Etched Foil").clicked() {
                                    self.decklist.add(top_card.card.clone(), FoilType::Etched);
                                }
                            });
                        });
                    });
                    ui.separator();
                    ui.label("Oracle text:");
                    ui.label(&top_card.card.faces[top_card.face_num as usize].oracle_text);

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add(egui::Image::new(format!(
                            "file://{}",
                            top_card
                                .card
                                .faces
                                .first()
                                .expect("unable to find card face image (corrupt cache?)")
                                .image_path
                                .to_string_lossy(),
                        )));
                    });
                }
            })
        });

        Panel::right("Deck Panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Decklist");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save").clicked()
                        && let Err(e) = self.save_deck()
                    {
                        eprintln!("Error saving deck: {e}");
                    }
                    if ui.button("Sort").clicked() {
                        self.decklist.sort();
                    }
                });
            });
            ScrollArea::new([false, true]).show(ui, |ui| {
                let mut remove_index: Option<usize> = None;
                for (i, dc) in self.decklist.cards.iter_mut().enumerate() {
                    if draw_deck_card(ui, dc) {
                        remove_index = Some(i);
                    }
                }
                if let Some(ri) = remove_index {
                    self.decklist.cards.remove(ri);
                }
            });
        });

        CentralPanel::default().show(ui, |ui| {
            ui.heading("Other matches");

            let mut new_card: Option<CachedCard> = None;
            let mut new_card_foiltype = FoilType::None;
            ScrollArea::new([false, true]).show(ui, |ui| {
                if !self.current_matched_cards.is_empty() {
                    for card in &self.current_matched_cards[1..] {
                        match draw_match_card(ui, &card.card, card.distance) {
                            Some(FoilType::Etched) => {
                                new_card = Some(card.card.clone());
                                new_card_foiltype = FoilType::Etched;
                            }
                            Some(FoilType::Foil) => {
                                new_card = Some(card.card.clone());
                                new_card_foiltype = FoilType::Foil;
                            }
                            Some(FoilType::None) => {
                                new_card = Some(card.card.clone());
                                new_card_foiltype = FoilType::None;
                            }
                            None => {}
                        }
                    }

                    if let Some(cached_card) = new_card {
                        self.decklist.add(cached_card, new_card_foiltype)
                    };
                };
            });
        });

        ui.request_repaint_after(Duration::from_millis(50));
    }
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Check recognition channel for new messages and update self accordingly
        if let Ok(r) = self.engine.recognition_rx.try_recv() {
            if let Some(texture) = &mut self.current_texture {
                texture.set(r.image, egui::TextureOptions::default());
            } else {
                self.current_texture =
                    Some(ui.load_texture("Camera Feed", r.image, egui::TextureOptions::default()));
            }

            // Only update match list if there is a card in frame.
            if !r.matches.is_empty() {
                self.current_matched_cards = r
                    .matches
                    .iter()
                    .map(|m| CardMatch {
                        distance: m.distance,
                        face_num: m.face,
                        card: self
                            .engine
                            .cache
                            .get(&m.card_id)
                            .expect("unable to find card id in cache. (corrupt cache?)")
                            .clone(),
                    })
                    .collect();
            }
        }

        match self.current_texture.as_ref() {
            Some(_) => self.draw_running_ui(ui),
            None => self.draw_loading_ui(ui),
        }
    }
}
