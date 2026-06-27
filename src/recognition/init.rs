use crossbeam::channel::Receiver;
use crossbeam::channel::{self, RecvTimeoutError};
use opencv::{Result, core::Mat};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crate::messages::{CameraFrame, RecognitionFrame};
use crate::recognition::image_proc::{detect_card, preprocess};

pub fn init_rec_thread(
    camera_rx: Receiver<CameraFrame>,
    is_running: Arc<AtomicBool>,
) -> (thread::JoinHandle<Result<()>>, Receiver<RecognitionFrame>) {
    let (tx, rx) = channel::bounded(2);

    let handle = thread::spawn(move || -> Result<()> {
        // Load cache here

        while is_running.load(Ordering::Relaxed) {
            match camera_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(camera_frame) => {
                    let mut display_frame = camera_frame.frame.clone();
                    let Ok(edges) = preprocess(&camera_frame.frame) else {
                        continue;
                    };

                    let mut warped_frame: Option<Mat> = None;
                    if let Some(card) = detect_card(&edges)? {
                        warped_frame = Some(card.warp(&display_frame)?);
                        card.draw(&mut display_frame)?;
                    };

                    // Database matching here

                    let result = RecognitionFrame {
                        display_frame,
                        warped_frame,
                        card_id: None,
                    };

                    let _ = tx.try_send(result);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
    });

    (handle, rx)
}
