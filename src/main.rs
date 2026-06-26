mod capture;
use opencv::{core::Mat, highgui, prelude::*, videoio};

use capture::Camera;

fn main() -> opencv::Result<()> {
    let mut camera = Camera::open(0)?;

    highgui::named_window("MagicVision", highgui::WINDOW_NORMAL)?;

    loop {
        let frame = camera.next_frame()?;

        highgui::imshow("MagicVision", &frame)?;
        if highgui::wait_key(1)? == 'q' as i32 {
            break;
        }
    }
    Ok(())
}
