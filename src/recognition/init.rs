use crossbeam::channel::Receiver;
use crossbeam::channel::{self, RecvTimeoutError};
use opencv::core::Mat;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    cache::{
        card_cache::{CardCache, load_card_cache},
        matching::{MatchDatabase, MatchResult, get_hasher},
    },
    messages::{CameraFrame, RecognitionFrame},
    recognition::{
        capture::init_cam_thread,
        image_proc::{detect_card, hash_mat, preprocess},
    },
    types::DynResult,
};

pub struct Engine {
    pub is_running: Arc<AtomicBool>,
    pub recognition_rx: Receiver<RecognitionFrame>,
    pub camera_handle: JoinHandle<DynResult<()>>,
    pub recognition_handle: JoinHandle<DynResult<()>>,
    pub cache: CardCache,
}
impl Engine {
    pub async fn start() -> DynResult<Self> {
        let cache = load_card_cache().await?;
        let match_db = MatchDatabase::from_cache(&cache);
        let is_running = Arc::new(AtomicBool::new(true));

        let (cam_handle, cam_rx) = init_cam_thread(is_running.clone());
        let (rec_handle, rec_rx) = init_rec_thread(cam_rx, is_running.clone(), match_db);

        Ok(Self {
            is_running,
            recognition_rx: rec_rx,
            camera_handle: cam_handle,
            recognition_handle: rec_handle,
            cache,
        })
    }

    pub fn stop(self) -> DynResult<()> {
        self.is_running.store(false, Ordering::Relaxed);
        self.camera_handle.join().unwrap()?;
        self.recognition_handle.join().unwrap()?;
        Ok(())
    }
}

pub fn init_rec_thread(
    cam_rx: Receiver<CameraFrame>,
    is_running: Arc<AtomicBool>,
    match_db: MatchDatabase,
) -> (JoinHandle<DynResult<()>>, Receiver<RecognitionFrame>) {
    let (tx, rx) = channel::bounded(2);

    let handle = thread::spawn(move || -> DynResult<()> {
        let hasher = get_hasher();

        while is_running.load(Ordering::Relaxed) {
            match cam_rx.recv_timeout(Duration::from_millis(50)) {
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
