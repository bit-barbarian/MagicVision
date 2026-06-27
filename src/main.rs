mod capture;
mod messages;
mod recognition;
use opencv::highgui;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{capture::init_cam_thread, recognition::init::init_rec_thread};

fn main() -> opencv::Result<()> {
    let is_running = Arc::new(AtomicBool::new(true));
    let (camera_handle, camera_rx) = init_cam_thread(is_running.clone());
    let (recognition_handle, recognition_rx) = init_rec_thread(camera_rx, is_running.clone());

    highgui::named_window("MagicVision", highgui::WINDOW_NORMAL)?;
    highgui::named_window("WarpFrame", highgui::WINDOW_NORMAL)?;

    loop {
        let Ok((result, warp_frame)) = recognition_rx.try_recv() else {
            continue;
        };

        highgui::imshow("MagicVision", &result.frame)?;
        if let Some(wf) = warp_frame {
            highgui::imshow("WarpFrame", &wf.frame)?;
        }
        if highgui::wait_key(1)? == 'q' as i32 {
            break;
        }
    }

    is_running.store(false, Ordering::Relaxed);
    camera_handle.join().unwrap()?;
    recognition_handle.join().unwrap()?;
    Ok(())
}
