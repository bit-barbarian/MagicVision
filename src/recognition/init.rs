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
                    let result = RecognitionFrame {
                        frame: camera_frame.frame,
                        card_id: None,
                    };

                    // Recognition logic here

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
