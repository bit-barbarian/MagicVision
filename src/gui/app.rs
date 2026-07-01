use std::time::Duration;

use egui::{CentralPanel, TextureHandle};

use crate::recognition::init::Engine;

pub struct Application {
    engine: Engine,
    cam_texture: Option<TextureHandle>,
}

impl Default for Application {
    fn default() -> Self {
        let engine = Engine::start().expect("Unable to start recognition engine.");
        Self {
            engine,
            cam_texture: None,
        }
    }
}

impl Application {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for Application {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| {
            if let Ok(recognition_frame) = self.engine.recognition_rx.try_recv() {
                if let Some(texture) = &mut self.cam_texture {
                    texture.set(recognition_frame.image, egui::TextureOptions::default())
                } else {
                    self.cam_texture = Some(ui.load_texture(
                        "Camera Feed",
                        recognition_frame.image,
                        egui::TextureOptions::default(),
                    ))
                };
            };

            if let Some(texture) = &self.cam_texture {
                ui.image(texture);
            };
        });

        ui.request_repaint_after(Duration::from_millis(50));
    }
}
