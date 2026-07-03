use egui::DragValue;

use crate::{
    cache::card_cache::CachedCard,
    gui::deck_file_formats::{DeckCard, FoilType},
};

pub fn draw_deck_card(ui: &mut egui::Ui, dc: &mut DeckCard) -> bool {
    let mut delete_card = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add(DragValue::new(&mut dc.count).range(1..=999).speed(0.05));
            ui.label(match dc.foil_type {
                FoilType::Etched => "Etched",
                FoilType::Foil => "Foil",
                FoilType::None => "Nonfoil",
            });
        });
        ui.vertical(|ui| {
            ui.heading(&dc.card.name);
            ui.horizontal(|ui| {
                ui.label(dc.card.set.to_uppercase());
                ui.label(&dc.card.number);
            });
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("-").clicked() {
                delete_card = true;
            }
            ui.add(
                egui::Image::new(format!(
                    "file://{}",
                    dc.card
                        .faces
                        .first()
                        .expect("unable to find card face image (corrupt cache?)")
                        .image_path
                        .to_string_lossy(),
                ))
                .sense(egui::Sense::hover()),
            )
            .on_hover_ui(|_ui| {
                ui.add(
                    egui::Image::new(format!(
                        "file://{}",
                        dc.card
                            .faces
                            .first()
                            .expect("unable to find card face image (corrupt cache?)")
                            .image_path
                            .to_string_lossy(),
                    ))
                    .fit_to_original_size(1.0),
                );
            });
        });
    });
    ui.separator();
    delete_card
}

pub fn draw_match_card(ui: &mut egui::Ui, card: &CachedCard, conf: u32) -> Option<FoilType> {
    let mut clicked: Option<FoilType> = None;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", conf));
                ui.label(&card.name);
            });
            ui.horizontal(|ui| {
                ui.label(card.set.to_uppercase());
                ui.label(&card.number);
            });
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.menu_button("+", |ui| {
                if ui.button("Add Nonfoil").clicked() {
                    clicked = Some(FoilType::None);
                }
                if ui.button("Add Foil").clicked() {
                    clicked = Some(FoilType::Foil);
                }
                if ui.button("Add Etched Foil").clicked() {
                    clicked = Some(FoilType::Etched);
                }
            });

            ui.add(
                egui::Image::new(format!(
                    "file://{}",
                    card.faces
                        .first()
                        .expect("unable to find card face image (corrupt cache?)")
                        .image_path
                        .to_string_lossy(),
                ))
                .sense(egui::Sense::hover()),
            )
            .on_hover_ui(|_ui| {
                ui.add(
                    egui::Image::new(format!(
                        "file://{}",
                        card.faces
                            .first()
                            .expect("unable to find card face image (corrupt cache?)")
                            .image_path
                            .to_string_lossy(),
                    ))
                    .fit_to_original_size(1.0),
                );
            });
        });
    });
    ui.separator();
    clicked
}
