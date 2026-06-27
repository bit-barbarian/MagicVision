use crossbeam::channel;
use crossbeam::channel::Receiver;
use opencv::{
    Result,
    prelude::*,
    videoio::{self, VideoCapture},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Instant,
};

use crate::messages::CameraFrame;

pub struct Camera {
    capture: VideoCapture,
}

impl Camera {
    pub fn open(index: i32) -> opencv::Result<Self> {
        let mut camera = VideoCapture::new(index, videoio::CAP_ANY)?;
        if !camera.is_opened()? {
            return Err(opencv::Error::new(1, "Unable to open camera!"));
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

pub fn init_cam_thread(
    is_running: Arc<AtomicBool>,
) -> (thread::JoinHandle<Result<()>>, Receiver<CameraFrame>) {
    let (tx, rx) = channel::bounded(2);

    let handle = thread::spawn(move || -> Result<()> {
        let mut camera = Camera::open(0)?;

        while is_running.load(Ordering::Relaxed) {
            let frame = camera.next_frame()?;

            let msg = CameraFrame {
                frame,
                timestamp: Instant::now(),
            };

            let _ = tx.try_send(msg);
        }
        Ok(())
    });

    (handle, rx)
}
