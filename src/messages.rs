use opencv::core::Mat;

pub struct CameraFrame {
    pub frame: Mat,
    pub timestamp: std::time::Instant,
}

pub struct RecognitionFrame {
    pub frame: Mat,
    pub card_id: Option<uuid::Uuid>,
}
