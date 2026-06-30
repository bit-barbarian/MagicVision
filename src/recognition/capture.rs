use crossbeam::channel;
use crossbeam::channel::Receiver;
use opencv::{
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

use crate::{
    constants::{CAMERA_HEIGHT, CAMERA_WIDTH},
    messages::CameraFrame,
    types::DynResult,
};

pub struct Camera {
    capture: VideoCapture,
}

impl Camera {
    fn open(index: i32) -> DynResult<Self> {
        let mut camera = VideoCapture::new(index, videoio::CAP_ANY)?;
        if !camera.is_opened()? {
            return Err("Unable to open camera!".into());
        }
        camera.set(videoio::CAP_PROP_FRAME_WIDTH, CAMERA_WIDTH)?;
        camera.set(videoio::CAP_PROP_FRAME_HEIGHT, CAMERA_HEIGHT)?;
        Ok(Camera { capture: camera })
    }

    fn next_frame(&mut self) -> opencv::Result<Mat> {
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
) -> (thread::JoinHandle<DynResult<()>>, Receiver<CameraFrame>) {
    let (tx, rx) = channel::bounded(2);

    let handle = thread::spawn(move || -> DynResult<()> {
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
