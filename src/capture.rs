use opencv::prelude::*;
use opencv::videoio::{self, VideoCapture};

pub struct Camera {
    capture: VideoCapture,
}

impl Camera {
    pub fn open(index: i32) -> opencv::Result<Self> {
        let mut camera = VideoCapture::new(index, videoio::CAP_ANY)?;
        if !camera.is_opened()? {
            panic!("Couldn't open camera");
        }
        camera.set(videoio::CAP_PROP_FRAME_WIDTH, 1920.0)?;
        camera.set(videoio::CAP_PROP_FRAME_HEIGHT, 1080.0)?;
        Ok(Camera { capture: camera })
    }

    pub fn next_frame(&mut self) -> opencv::Result<Mat> {
        let mut frame = Mat::default();
        self.capture.read(&mut frame)?;

        if frame.empty() {
            Err(opencv::Error::new(1, "Frame empty!"))
        } else {
            Ok(frame)
        }
    }
}
