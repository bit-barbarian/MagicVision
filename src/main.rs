mod cache;
mod capture;
mod constants;
mod messages;
mod recognition;
mod types;
use crossbeam::channel::RecvTimeoutError;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use opencv::highgui;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crate::{
    cache::{card_cache::load_card_cache, matching::MatchDatabase},
    capture::init_cam_thread,
    recognition::init::init_rec_thread,
    types::DynResult,
};

#[tokio::main]
async fn main() -> DynResult<()> {
    let cache = load_card_cache().await?;
    let match_db = MatchDatabase::from_cache(&cache);

    let is_running = Arc::new(AtomicBool::new(true));
    let (camera_handle, camera_rx) = init_cam_thread(is_running.clone());
    let (recognition_handle, recognition_rx) =
        init_rec_thread(camera_rx, is_running.clone(), match_db);

    highgui::named_window("MagicVision", highgui::WINDOW_NORMAL)?;
    highgui::named_window("WarpFrame", highgui::WINDOW_NORMAL)?;

    loop {
        match recognition_rx.recv_timeout(Duration::from_millis(10)) {
            Ok(result) => {
                highgui::imshow("MagicVision", &result.display_frame)?;
                if let Some(wf) = result.warped_frame {
                    highgui::imshow("WarpFrame", &wf)?;
                }
                if highgui::wait_key(1)? == 'q' as i32 {
                    break;
                }

                for (i, m) in result.matches.iter().enumerate() {
                    if let Some(card) = cache.get(&m.card_id) {
                        println!("Match #{}: {} ({} {})", i, card.name, card.set, card.number);
                        println!("Distance:  {}", m.distance);
                    };
                }

                execute!(io::stdout(), Clear(ClearType::All))?;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
    }

    is_running.store(false, Ordering::Relaxed);
    camera_handle.join().unwrap()?;
    recognition_handle.join().unwrap()?;
    Ok(())
}
