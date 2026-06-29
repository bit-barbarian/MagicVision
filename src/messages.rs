use opencv::core::Mat;

use crate::cache::matching::MatchResult;

pub struct CameraFrame {
    pub frame: Mat,
    pub timestamp: std::time::Instant,
}

pub struct RecognitionFrame {
    pub display_frame: Mat,
    pub warped_frame: Option<Mat>,
    pub matches: Vec<MatchResult>,
}
