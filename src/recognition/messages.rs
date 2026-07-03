use egui::ColorImage;
use opencv::core::Mat;

use crate::cache::matching::MatchResult;

pub struct CameraFrame {
    pub frame: Mat,
    pub timestamp: std::time::Instant,
}

pub struct RecognitionFrame {
    pub image: ColorImage,
    pub matches: Vec<MatchResult>,
}
