use crossbeam::channel::Receiver;
use crossbeam::channel::{self, RecvTimeoutError};
use opencv::Result;
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
) -> (
    thread::JoinHandle<Result<()>>,
    Receiver<(RecognitionFrame, Option<RecognitionFrame>)>,
) {
    let (tx, rx) = channel::bounded(2);

    let handle = thread::spawn(move || -> Result<()> {
        // Load cache here

        while is_running.load(Ordering::Relaxed) {
            match camera_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(camera_frame) => {
                    let mut display_frame = camera_frame.frame.clone();
                    // Recognition logic here
                    let Ok(edges) = preprocess(&camera_frame.frame) else {
                        continue;
                    };

                    let mut warp_result: Option<RecognitionFrame> = None;

                    if let Some(card) = detect_card(&edges)? {
                        warp_result = Some(RecognitionFrame {
                            frame: card.warp(&display_frame)?,
                            card_id: None,
                        });
                        card.draw(&mut display_frame)?;
                    };

                    let result = RecognitionFrame {
                        frame: display_frame,
                        card_id: None,
                    };

                    let _ = tx.try_send((result, warp_result));
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
    });

    (handle, rx)
}
