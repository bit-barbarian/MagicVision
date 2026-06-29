use crossbeam::channel::Receiver;
use crossbeam::channel::{self, RecvTimeoutError};
use opencv::core::Mat;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crate::cache::matching::MatchResult;
use crate::{
    cache::matching::{MatchDatabase, get_hasher},
    messages::{CameraFrame, RecognitionFrame},
    recognition::image_proc::{detect_card, hash_mat, preprocess},
    types::DynResult,
};

pub fn init_rec_thread(
    camera_rx: Receiver<CameraFrame>,
    is_running: Arc<AtomicBool>,
    match_db: MatchDatabase,
) -> (
    thread::JoinHandle<DynResult<()>>,
    Receiver<RecognitionFrame>,
) {
    let (tx, rx) = channel::bounded(2);

    let handle = thread::spawn(move || -> DynResult<()> {
        let hasher = get_hasher();

        while is_running.load(Ordering::Relaxed) {
            match camera_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(camera_frame) => {
                    // Preprocess camera frame
                    let mut display_frame = camera_frame.frame.clone();
                    let Ok(edges) = preprocess(&camera_frame.frame) else {
                        continue;
                    };

                    // Detect where card is in frame
                    let mut warped_frame: Option<Mat> = None;
                    let mut matches: Vec<MatchResult> = Vec::new();
                    if let Some(card) = detect_card(&edges)? {
                        warped_frame = Some(card.warp(&display_frame)?);
                        card.draw(&mut display_frame)?;
                    };

                    // Database matching
                    if let Some(wf) = &warped_frame {
                        let hash = hash_mat(wf, &hasher)?;
                        // scan db and make list of top 10 closest hashes by hamming distance
                        matches = match_db.best_matches(&hash);
                    }

                    let result = RecognitionFrame {
                        display_frame,
                        warped_frame,
                        matches,
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
